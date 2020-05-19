#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel_migrations;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use rocket_contrib::json::Json;
use std::env;

embed_migrations!();

mod api_schemas;
mod passwords;

#[post("/register", data = "<registration>")]
fn register(
    registration: Json<api_schemas::RegistrationInfo>,
) -> Json<api_schemas::RegistrationInfoResponse> {
    let password_hash = passwords::hash_password(&registration.password)
        .unwrap()
        .to_string();

    Json(api_schemas::RegistrationInfoResponse {
        email: registration.email.to_string(),
        display_name: registration.display_name.to_string(),
    })
}

fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish("./data.db")
        .expect(&format!("Error connecting to {}", database_url))
}

fn main() {
    let connection = establish_connection();
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout())
        .expect("Error applying database migrations");

    rocket::ignite().mount("/", routes![register]).launch();
}
