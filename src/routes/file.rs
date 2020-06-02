use crate::api_error::{ApiError, CustomError};
use crate::models::{FilePath, Message, UploadID, User, PendingUpload};
use crate::PendingUploadStore;
use rocket::data::Data;
use rocket::http::Status;
use rocket::State;
use rocket_contrib::json::Json;
use std::path::{Component, PathBuf};
use uuid::Uuid;

/// Prepare a new file upload to the server
///
/// This is needed because saving the entire multipart data to a temporary
/// location before reading its fields and moving the file to the correct
/// path is undesirable. This route returns an upload ID which can be used
/// to upload a file at a given path.
///
/// Paths must be absolute. The root represents the user's content storage
/// root. Relative paths are rejected. Paths may not contain references to
/// the parent directory. Paths must also point to a file and not a directory.
#[post("/upload/new", data = "<path>")]
pub fn new_upload(
    path: Json<FilePath>,
    user: User,
    pending_uploads: State<PendingUploadStore>,
) -> Result<Json<UploadID>, ApiError> {
    let path = PathBuf::from(path.into_inner().path);
    if path.components().any(|c| c == Component::ParentDir) {
        Err(CustomError::new(
            "Paths referencing the parent dir are not allowed",
            Status::BadRequest,
        ))?;
    }
    if path.is_relative() {
        Err(CustomError::new(
            "Relative paths are not allowed",
            Status::BadRequest,
        ))?;
    }
    if path.is_dir() {
        Err(CustomError::new(
            "Paths must point to a file",
            Status::BadRequest,
        ))?;
    }

    let upload_id = Uuid::new_v4();
    let pending_upload = PendingUpload {path, user};
    pending_uploads.write().insert(upload_id, pending_upload);

    Ok(Json(UploadID {upload_id}))
}

#[post("/upload/<id>", data = "<file>")]
pub fn upload(id: u32, user: User, file: Data) -> Result<Json<Message>, ApiError> {
    // TODO: Ensure the ID is valid
    // TODO: Ensure user that was given this ID is the user making the request
    // TODO: Stream data to that location
    Ok(Json(Message {
        message: "Upload successful".to_string(),
    }))
}
