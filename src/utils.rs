use crate::api_error::ApiError;
use crate::models::user::User;
use std::env;
use std::path::PathBuf;

pub fn user_root_path(user: &User) -> Result<PathBuf, ApiError> {
    let storage_root = env::var("STORAGE_LOCATION").map_err(|_| ApiError::InternalServerError)?;

    Ok(PathBuf::from(format!("{}/{}", storage_root, user.id)))
}