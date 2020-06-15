use crate::api_error::{ApiError, CustomError};
use crate::models::user::User;
use crate::schema::shares;
use rocket::http::Status;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Component, PathBuf};
use std::time::Instant;

#[derive(Deserialize)]
pub struct JsonPath {
    pub path: String,
}

impl JsonPath {
    pub fn to_pathbuf(self) -> Result<PathBuf, ApiError> {
        let path = PathBuf::from(self.path);
        if path.components().any(|c| c == Component::ParentDir) {
            Err(CustomError::new(
                "Paths referencing the parent dir are not allowed".to_string(),
                Status::BadRequest,
            ))?;
        }
        if path.is_absolute() {
            Err(CustomError::new(
                "Absolute paths are not allowed".to_string(),
                Status::BadRequest,
            ))?;
        }

        Ok(path)
    }
}

#[derive(Serialize)]
pub struct UploadID {
    pub upload_id: uuid::Uuid,
}

#[derive(Clone)]
pub struct PendingUpload {
    pub created: Instant,
    pub path: PathBuf,
    pub user: User,
}

#[derive(Debug, Serialize)]
pub enum FileSystemElementType {
    File,
    Directory,
}

#[derive(Debug, Serialize)]
pub struct FileSystemElement {
    pub element_type: FileSystemElementType,
    pub name: String,
    pub bytes: u64,
}

#[derive(Serialize)]
pub struct DirContents {
    pub contents: Vec<FileSystemElement>,
}

impl fmt::Debug for DirContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.contents.iter()).finish()
    }
}

#[table_name = "shares"]
#[derive(Insertable, Queryable, Serialize)]
pub struct Share {
    pub link: String,
    pub path: String
}
