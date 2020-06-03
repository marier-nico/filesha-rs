use crate::api_error::{ApiError, CustomError};
use crate::schema::users;
use diesel::{Insertable, Queryable};
use rocket::http::Status;
use serde;
use std::cmp::PartialEq;
use std::fmt;
use std::path::{Component, PathBuf};

#[derive(PartialEq, serde::Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[table_name = "users"]
#[derive(serde::Deserialize, Insertable)]
pub struct UserCreate {
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(serde::Serialize, Queryable)]
pub struct UserResult {
    pub id: i32,
    pub email: String,
    pub display_name: String,
}

impl UserResult {
    pub fn from(user: &User) -> UserResult {
        UserResult {
            id: user.id,
            email: user.email.to_string(),
            display_name: user.display_name.to_string(),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct Message {
    pub message: String,
}

#[derive(serde::Deserialize)]
pub struct JsonPath {
    pub path: String,
}

impl JsonPath {
    pub fn to_pathbuf(self) -> Result<PathBuf, ApiError> {
        let path = PathBuf::from(self.path);
        if path.components().any(|c| c == Component::ParentDir) {
            Err(CustomError::new(
                "Paths referencing the parent dir are not allowed",
                Status::BadRequest,
            ))?;
        }
        if path.is_absolute() {
            Err(CustomError::new(
                "Absolute paths are not allowed",
                Status::BadRequest,
            ))?;
        }

        Ok(path)
    }
}

#[derive(serde::Serialize)]
pub struct UploadID {
    pub upload_id: uuid::Uuid,
}

pub struct PendingUpload {
    pub path: PathBuf,
    pub user: User,
}

#[derive(Debug, serde::Serialize)]
pub enum FileSystemElementType {
    File,
    Directory,
}

#[derive(Debug, serde::Serialize)]
pub struct FileSystemElement {
    pub element_type: FileSystemElementType,
    pub name: String,
    pub bytes: u64,
}

#[derive(serde::Serialize)]
pub struct DirContents {
    pub contents: Vec<FileSystemElement>,
}

impl fmt::Debug for DirContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.contents.iter()).finish()
    }
}