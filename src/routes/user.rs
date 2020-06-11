use crate::api_error::{ApiError, CustomError};
use crate::db;
use crate::models::user::{ActiveSession, UserCreate, UserLogin, UserResult};
use crate::passwords;
use crate::{DBConnection, SessionStore};
use rocket::http::{Cookie, Cookies, Status};
use rocket::State;
use rocket_contrib::json::Json;
use std::env;
use std::time::Instant;
use uuid::Uuid;

#[post("/register", data = "<user>")]
pub fn register(
    conn: DBConnection,
    user: Json<UserCreate>,
    active_session_ids: State<SessionStore>,
    mut cookies: Cookies,
) -> Result<Json<UserResult>, ApiError> {
    env::var("ALLOW_REGISTRATIONS").map_err(|_| {
        CustomError::new(
            "User registrations have been disabled".to_string(),
            Status::Unauthorized,
        )
    })?;

    let mut user = user.into_inner();
    let password_hash = passwords::hash_password(&user.password)?.to_string();
    user.password = password_hash;

    db::user::create(&user, &*conn)?;
    let created_user = db::user::get_by_email(&user.email, &*conn)?
        .ok_or_else(|| ApiError::InternalServerError)?;

    let session_id = Uuid::new_v4();
    cookies.add_private(Cookie::new("session", session_id.to_string()));
    let mut session_ids = active_session_ids.write();
    session_ids.insert(
        session_id,
        ActiveSession {
            email: user.email,
            created: Instant::now(),
        },
    );

    Ok(Json(UserResult::from(&created_user)))
}

/// Logs in a user if they provide valid credentials.
///
/// A key component of this login route is that it operates in constant-time,
/// meaning that it takes the same amount of time to return a response irrespective
/// of whether or not the user exists. Verifying if passwords match takes time, but
/// finding out that a user does not exist in the database is very fast, so we have
/// to simulate a delay to ensure that no information can be gathered from the timing
/// of the responses from this route.
#[post("/login", data = "<user>")]
pub fn login(
    conn: DBConnection,
    user: Json<UserLogin>,
    active_session_ids: State<SessionStore>,
    mut cookies: Cookies,
) -> Result<Json<UserResult>, ApiError> {
    let user = user.into_inner();
    let db_user = db::user::get_by_email(&user.email, &*conn)?.ok_or_else(|| {
        match passwords::hash_password(&"Dummy Password") {
            Ok(_) => (),
            Err(_) => return ApiError::InternalServerError,
        }
        ApiError::from(CustomError::new(
            "User not found or incorrect password".to_string(),
            Status::BadRequest,
        ))
    })?;

    let password_hash = passwords::PasswordHash::from(&db_user.password)?;
    passwords::verify_password(&user.password, &password_hash).map_err(|_| {
        CustomError::new(
            "User not found or incorrect password".to_string(),
            Status::BadRequest,
        )
    })?;

    let session_id = Uuid::new_v4();
    cookies.add_private(Cookie::new("session", session_id.to_string()));
    let mut active_session_ids = active_session_ids.write();
    active_session_ids.insert(
        session_id,
        ActiveSession {
            email: db_user.email.to_string(),
            created: Instant::now(),
        },
    );

    Ok(Json(UserResult::from(&db_user)))
}
