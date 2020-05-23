use crate::DBConnection;
use crate::models;
use crate::passwords;
use crate::generate_error;
use crate::schema;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::json::Json;


#[post("/register", data = "<user>")]
pub fn register(
    conn: DBConnection,
    user: Json<models::User>,
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

    Ok(Json(models::UserResult {
        email: user.email.to_string(),
        display_name: user.display_name.to_string(),
    }))
}