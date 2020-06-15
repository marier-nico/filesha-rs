use crate::api_error::{ApiError, CustomError};
use crate::db;
use crate::models::common_models::Message;
use crate::models::file::{
    DirContents, FileSystemElement, FileSystemElementType, JsonPath, PendingUpload, Share, UploadID,
};
use crate::models::user::User;
use crate::utils;
use crate::DBConnection;
use crate::PendingUploadStore;
use rocket::data::Data;
use rocket::http::Status;
use rocket::response::NamedFile;
use rocket::State;
use rocket_contrib::json::Json;
use std::fs;
use std::path::Path;
use std::time::Instant;
use tempfile::tempdir;
use uuid::Uuid;

/// Prepare a new file upload to the server
///
/// This is needed because saving the entire multipart data to a temporary
/// location before reading its fields and moving the file to the correct
/// path is undesirable. This route returns an upload ID which can be used
/// to upload a file at a given path.
///
/// Paths must be relative to the root of the user's content storage directory.
/// Absolute paths are rejected. Paths may not contain references to the parent
/// directory. Paths must also point to a file and not a directory.
#[post("/upload/new", data = "<path>")]
pub fn new_upload(
    path: Json<JsonPath>,
    user: User,
    pending_uploads: State<PendingUploadStore>,
) -> Result<Json<UploadID>, ApiError> {
    let path = utils::user_root_path(&user)?.join(path.into_inner().to_pathbuf()?);
    if path.is_dir() {
        Err(CustomError::new(
            "Paths must point to a file".to_string(),
            Status::BadRequest,
        ))?;
    }

    let upload_id = Uuid::new_v4();
    let pending_upload = PendingUpload {
        path,
        user,
        created: Instant::now(),
    };
    pending_uploads.write().insert(upload_id, pending_upload);

    Ok(Json(UploadID { upload_id }))
}

#[post("/upload/<id>", data = "<file>")]
pub fn upload(
    id: String,
    user: User,
    file: Data,
    pending_uploads_lock: State<PendingUploadStore>,
) -> Result<Json<Message>, ApiError> {
    let parsed_id = Uuid::parse_str(&id)
        .map_err(|_| CustomError::new("Invalid upload ID".to_string(), Status::BadRequest))?;
    let pending_uploads = pending_uploads_lock.read();
    let associated_upload = pending_uploads
        .get(&parsed_id)
        .ok_or_else(|| CustomError::new("Upload ID not in use".to_string(), Status::BadRequest))?;
    if associated_upload.user != user {
        Err(CustomError::new(
            "A different user created this upload".to_string(),
            Status::Unauthorized,
        ))?;
    }

    let str_path = associated_upload
        .path
        .to_str()
        .ok_or_else(|| ApiError::InternalServerError)?
        .to_string();

    if let Some(path) = associated_upload.path.parent() {
        fs::create_dir_all(path)
            .map_err(|e| CustomError::new(e.to_string(), Status::InternalServerError))?;
    }
    drop(pending_uploads); // Release the lock explicitly before streaming the file to disk
    file.stream_to_file(str_path)
        .map_err(|e| CustomError::new(e.to_string(), Status::InternalServerError))?;

    let mut pending_uploads = pending_uploads_lock.write();
    pending_uploads.remove(&parsed_id);

    Ok(Json(Message {
        message: "Upload successful".to_string(),
    }))
}

#[post("/ls", data = "<path>")]
pub fn ls(path: Json<JsonPath>, user: User) -> Result<Json<DirContents>, ApiError> {
    let path = utils::user_root_path(&user)?.join(path.into_inner().to_pathbuf()?);
    let mut contents = vec![];
    let entries =
        fs::read_dir(path).map_err(|e| CustomError::new(e.to_string(), Status::BadRequest))?;
    for entry in entries {
        let entry = entry?;
        let metadata = entry.metadata()?;

        let (fs_element_type, fs_element_size) = {
            if metadata.is_dir() {
                (FileSystemElementType::Directory, 0)
            } else {
                (FileSystemElementType::File, metadata.len())
            }
        };
        let fs_element = FileSystemElement {
            element_type: fs_element_type,
            name: entry.file_name().to_string_lossy().to_string(),
            bytes: fs_element_size,
        };
        contents.push(fs_element);
    }

    Ok(Json(DirContents { contents }))
}

#[post("/mkdir", data = "<path>")]
pub fn mkdir(path: Json<JsonPath>, user: User) -> Result<Json<Message>, ApiError> {
    let path = utils::user_root_path(&user)?.join(path.into_inner().to_pathbuf()?);

    fs::create_dir_all(path)
        .map_err(|e| CustomError::new(e.to_string(), Status::InternalServerError))?;

    Ok(Json(Message {
        message: "Directory created successfully".to_string(),
    }))
}

#[post("/download", data = "<path>")]
pub fn download(path: Json<JsonPath>, user: User) -> Result<NamedFile, ApiError> {
    let path = utils::user_root_path(&user)?.join(path.into_inner().to_pathbuf()?);

    get_named_file(&path)
}

#[post("/share", data = "<path>")]
pub fn create_share(
    path: Json<JsonPath>,
    user: User,
    conn: DBConnection,
) -> Result<Json<Share>, ApiError> {
    let user_prefix = utils::user_root_path(&user)?;
    let user_path = path.into_inner().to_pathbuf()?;
    let full_path = user_prefix.join(&user_path);

    let share = Share {
        link: Uuid::new_v4().to_string(),
        path: full_path
            .to_str()
            .ok_or_else(|| ApiError::InternalServerError)?
            .to_string(),
    };
    db::file::save_share(&share, &conn)?;

    // Return just the user path to avoid revealing the user's ID
    let returned_share = Share {
        link: share.link.clone(),
        path: user_path
            .to_str()
            .ok_or_else(|| ApiError::InternalServerError)?
            .to_string(),
    };
    Ok(Json(returned_share))
}

#[get("/shared/<id>")]
pub fn download_shared(id: String, conn: DBConnection) -> Result<NamedFile, ApiError> {
    let share = db::file::get_share(&id, &conn)?.ok_or_else(|| {
        CustomError::new(
            "This share ID does not exist".to_string(),
            Status::BadRequest,
        )
    })?;

    let path = Path::new(&share.path);
    get_named_file(path)
}

fn get_named_file(path: &Path) -> Result<NamedFile, ApiError> {
    if path.is_dir() {
        let temp_dir =
            tempdir().map_err(|e| CustomError::new(e.to_string(), Status::InternalServerError))?;
        let file_path = temp_dir
            .path()
            .join(format!("{:?}.zip", path.file_name().unwrap()));
        let zip_file = fs::File::create(&file_path)
            .map_err(|e| CustomError::new(e.to_string(), Status::InternalServerError))?;

        utils::zip_dir_recursive(path, &zip_file)
            .map_err(|e| CustomError::new(e.to_string(), Status::InternalServerError))?;
        let file = NamedFile::open(file_path)
            .map_err(|e| CustomError::new(e.to_string(), Status::BadRequest))?;
        Ok(file)
    } else {
        let file = NamedFile::open(path)
            .map_err(|e| CustomError::new(e.to_string(), Status::BadRequest))?;
        Ok(file)
    }
}
