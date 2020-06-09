use crate::api_error::ApiError;
use crate::models::user::User;
use std::env;
use std::path::PathBuf;

pub fn user_root_path(user: &User) -> Result<PathBuf, ApiError> {
    let storage_root = env::var("STORAGE_LOCATION").unwrap();

    Ok(PathBuf::from(format!("{}/{}", storage_root, user.id)))
}

pub fn ensure_all_env_vars_are_set() -> Result<(), ApiError> {
    let vars = ["DATABASE_URL", "ROCKET_DATABASES", "STORAGE_LOCATION"];
    let missing: Vec<&&str> = vars.iter().filter(|v| env::var(v).is_err()).collect();

    if !missing.is_empty() {
        return Err(ApiError::MissingEnvVars(missing.iter().map(|v| v.to_string()).collect()))
    }

    Ok(())
}