use crate::api_error::ApiError;
use crate::models::file::PendingUpload;
use crate::models::user::{ActiveSession, User};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::{PathBuf, Path};
use std::time::{Duration, Instant};
use uuid::Uuid;
use walkdir::WalkDir;
use zip::result::{ZipError, ZipResult};
use zip::write::{FileOptions, ZipWriter};

const TINY_FILE_BUF_SIZE: usize = 8 * 1024;
const SMALL_FILE_BUF_SIZE: usize = 16 * 1024;
const MEDIUM_FILE_SIZE: usize = 256 * 1024;
const LARGE_FILE_BIF_SIZE: usize = 2 * 1024 * 1024;

pub fn user_root_path(user: &User) -> Result<PathBuf, ApiError> {
    let storage_root = env::var("STORAGE_LOCATION").unwrap();

    Ok(PathBuf::from(format!("{}/{}", storage_root, user.id)))
}

pub fn ensure_all_env_vars_are_set() -> Result<(), ApiError> {
    let vars = ["DATABASE_URL", "ROCKET_DATABASES", "STORAGE_LOCATION"];
    let missing: Vec<&&str> = vars.iter().filter(|v| env::var(v).is_err()).collect();

    if !missing.is_empty() {
        return Err(ApiError::MissingEnvVars(
            missing.iter().map(|v| v.to_string()).collect(),
        ));
    }

    Ok(())
}

pub fn remove_old_pending_uploads(
    pending_uploads: &HashMap<Uuid, PendingUpload>,
) -> HashMap<Uuid, PendingUpload> {
    let seconds_in_a_day = 60 * 60 * 24; // Keep pending uploads for one day before getting rid of them
    pending_uploads
        .into_iter()
        .filter(|(_uuid, pending_upload)| {
            let duration_since = Instant::now().duration_since(pending_upload.created);
            duration_since < Duration::new(seconds_in_a_day, 0)
        })
        .map(|(uuid, v)| (uuid.clone(), v.clone()))
        .collect()
}

pub fn remove_old_sessions(
    active_sessions: &HashMap<Uuid, ActiveSession>,
) -> HashMap<Uuid, ActiveSession> {
    let seconds_in_a_day = 60 * 60 * 24 * 7; // Keep sessions for one week before getting rid of them
    active_sessions
        .into_iter()
        .filter(|(_uuid, pending_upload)| {
            let duration_since = Instant::now().duration_since(pending_upload.created);
            duration_since < Duration::new(seconds_in_a_day, 0)
        })
        .map(|(uuid, v)| (uuid.clone(), v.clone()))
        .collect()
}

pub fn zip_dir_recursive(source_dir: &Path, destination_file: &File) -> ZipResult<()> {
    if !source_dir.is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let mut zip = ZipWriter::new(destination_file);
    zip.set_comment("filesha-rs");
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for entry in WalkDir::new(&source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(source_dir).unwrap();

        if path.is_file() {
            let mut file = File::open(path)?;
            let file_size = file.metadata()?.len();
            let buf_size = {
                if file_size <= 100 * 1024 {
                    TINY_FILE_BUF_SIZE
                } else if file_size <= 1024 * 1024 {
                    SMALL_FILE_BUF_SIZE
                } else if file_size <= 40 * 1024 * 1024 {
                    MEDIUM_FILE_SIZE
                } else {
                    LARGE_FILE_BIF_SIZE
                }
            };

            zip.start_file_from_path(name, options)?;
            let mut buf: Vec<u8> = Vec::with_capacity(buf_size);
            unsafe { buf.set_len(buf_size) }
            loop {
                let _len = match file.read(&mut buf) {
                    Ok(0) => break,
                    Ok(len) => len,
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(e) => return Err(ZipError::Io(e)),
                };

                zip.write_all(&buf)?;
            }
        } else if name.as_os_str().len() != 0 {
            zip.add_directory_from_path(name, options)?;
        }
    }

    zip.finish()?;
    Ok(())
}
