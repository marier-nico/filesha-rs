use crate::generate_error;
use crate::models;
use crate::passwords;
use crate::schema;
use crate::DBConnection;
use diesel::prelude::*;
use rocket::http::{Status, Cookie, Cookies};
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
        .load::<models::User>(&*conn)
        .expect("Error finding user");

    cookies.add_private(Cookie::new("session", (&result[0]).id.to_string()));
    let mut session_ids = active_session_ids.write();
    session_ids.insert((&result[0]).id);
    
    Ok(Json(models::UserResult::from(&result[0])))
}
