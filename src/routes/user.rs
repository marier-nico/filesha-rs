use crate::generate_error;
use crate::models;
use crate::passwords;
use crate::schema;
use crate::DBConnection;
use diesel::prelude::*;
use rocket::http::{Cookie, Cookies, Status};
use rocket::response::status;
use rocket::State;
use rocket_contrib::json::Json;

#[post("/register", data = "<user>")]
pub fn register(
    conn: DBConnection,
    user: Json<models::UserCreate>,
    active_session_ids: State<crate::SessionStore>,
    mut cookies: Cookies,
) -> Result<Json<models::UserResult>, status::Custom<Json<models::ErrorResponse>>> {
    let mut user = user.into_inner();
    let password_hash = match passwords::hash_password(&user.password) {
        Ok(hash) => hash,
        Err(_) => {
            return Err(generate_error(
                "Could not encrypt password".to_string(),
                Status::InternalServerError,
            ))
        }
    }
    .to_string();

    user.password = password_hash;

    if let Err(_) = diesel::insert_into(schema::users::table)
        .values(&user)
        .execute(&*conn)
    {
        return Err(generate_error(
            "This user is already registered".to_string(),
            Status::BadRequest,
        ));
    }

    let result = schema::users::table
        .filter(schema::users::email.eq(user.email))
        .limit(1)
        .load::<models::User>(&*conn);
    let result = match result {
        Ok(users) => users,
        Err(_) => {
            return Err(generate_error(
                "The user could not be created in the database".to_string(),
                Status::InternalServerError,
            ))
        }
    };

    cookies.add_private(Cookie::new("session", (&result[0]).id.to_string()));
    let mut session_ids = active_session_ids.write();
    session_ids.insert((&result[0]).id);

    Ok(Json(models::UserResult::from(&result[0])))
}

#[post("/login", data = "<user>")]
pub fn login(
    conn: DBConnection,
    user: Json<models::UserLogin>,
    active_session_ids: State<crate::SessionStore>,
    mut cookies: Cookies,
) -> Result<Json<models::UserResult>, status::Custom<Json<models::ErrorResponse>>> {
    let user = user.into_inner();
    let query_result = schema::users::table
        .filter(schema::users::email.eq(user.email))
        .limit(1)
        .load::<models::User>(&*conn);
    let db_user = match query_result {
        Ok(users) => users.into_iter().next(),
        Err(_) => {
            return Err(generate_error(
                "Error reading users in the database".to_string(),
                Status::BadRequest,
            ));
        }
    };
    let db_user = match db_user {
        Some(user) => user,
        None => {
            return Err(generate_error(
                "Email not found, or incorrect password".to_string(),
                Status::BadRequest,
            ))
        }
    };

    let password_hash = match passwords::PasswordHash::from(&db_user.password) {
        Ok(hash) => hash,
        Err(e) => return Err(generate_error(e.to_string(), Status::InternalServerError)),
    };
    if let Err(_) = passwords::verify_password(&user.password, &password_hash) {
        return Err(generate_error(
            "Email not found, or incorrect password".to_string(),
            Status::BadRequest,
        ));
    }

    cookies.add_private(Cookie::new("session", db_user.id.to_string()));
    let mut active_session_ids = active_session_ids.write();
    active_session_ids.insert(db_user.id);

    Ok(Json(models::UserResult::from(&db_user)))
}
