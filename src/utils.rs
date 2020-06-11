use crate::api_error::ApiError;
use crate::models::file::PendingUpload;
use crate::models::user::User;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use uuid::Uuid;

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
