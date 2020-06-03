use crate::api_error::{ApiError, CustomError};
use crate::models::{
    DirContents, FileSystemElement, FileSystemElementType, JsonPath, Message, PendingUpload,
    UploadID, User,
};
use crate::PendingUploadStore;
use rocket::data::Data;
use rocket::http::Status;
use rocket::State;
use rocket_contrib::json::Json;
use std::env;
use std::fs;
use std::path::PathBuf;
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
    let path = path.into_inner().to_pathbuf()?;
    let storage_root = env::var("STORAGE_LOCATION").map_err(|_| ApiError::InternalServerError)?;
    let full_path = PathBuf::from(format!("{}/{}", storage_root, user.id)).join(path);
    if full_path.is_dir() {
        Err(CustomError::new(
            "Paths must point to a file",
            Status::BadRequest,
        ))?;
    }

    let upload_id = Uuid::new_v4();
    let pending_upload = PendingUpload {
        path: full_path,
        user,
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
        .map_err(|_| CustomError::new("Invalid upload ID", Status::BadRequest))?;
    let pending_uploads = pending_uploads_lock.read();
    let associated_upload = pending_uploads
        .get(&parsed_id)
        .ok_or_else(|| CustomError::new("Upload ID not in use", Status::BadRequest))?;
    if associated_upload.user != user {
        Err(CustomError::new(
            "A different user created this upload",
            Status::Unauthorized,
        ))?;
    }

    let str_path = associated_upload
        .path
        .to_str()
        .ok_or_else(|| ApiError::InternalServerError)?
        .to_string();

    if let Some(path) = associated_upload.path.parent() {
        fs::create_dir_all(path).map_err(|_| {
            CustomError::new(
                "Could not create dir to save file",
                Status::InternalServerError,
            )
        })?;
    }
    drop(pending_uploads); // Release the lock explicitly before streaming the file to disk
    file.stream_to_file(str_path).map_err(|_| {
        CustomError::new("Could not save data to file", Status::InternalServerError)
    })?;

    let mut pending_uploads = pending_uploads_lock.write();
    pending_uploads.remove(&parsed_id);

    Ok(Json(Message {
        message: "Upload successful".to_string(),
    }))
}

#[post("/ls", data = "<path>")]
pub fn ls(path: Json<JsonPath>, user: User) -> Result<Json<DirContents>, ApiError> {
    let path = path.into_inner().to_pathbuf()?;
    let storage_root = env::var("STORAGE_LOCATION").map_err(|_| ApiError::InternalServerError)?;
    let full_path = PathBuf::from(format!("{}/{}", storage_root, user.id)).join(path);
    if !full_path.is_dir() {
        Err(CustomError::new(
            "Path must be a directory",
            Status::BadRequest,
        ))?;
    }

    let mut contents = vec![];
    let entries = fs::read_dir(full_path).map_err(|_| {
        CustomError::new(
            "The directory does not exist, or you are not allowed to view it",
            Status::BadRequest,
        )
    })?;
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
