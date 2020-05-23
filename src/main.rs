#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use diesel::prelude::*;
use dotenv::dotenv;
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::databases::diesel::SqliteConnection;
use rocket_contrib::json::Json;
use parking_lot::RwLock;
use std::env;
use std::collections::HashSet;

embed_migrations!();

mod models;
mod passwords;
mod schema;
mod routes {
    pub mod user;
}

type SessionStore = RwLock<HashSet<i32>>;

#[database("data_db")]
pub struct DBConnection(SqliteConnection);

#[catch(500)]
fn server_error(_req: &rocket::Request) -> Json<models::ErrorResponse> {
    Json(models::ErrorResponse {
        message: "The server encountered an error processing your request".to_string(),
    })
}

#[catch(404)]
fn not_found(_req: &rocket::Request) -> Json<models::ErrorResponse> {
    Json(models::ErrorResponse {
        message: "Not found".to_string(),
    })
}

pub fn generate_error(
    message: String,
    status: Status,
) -> status::Custom<Json<models::ErrorResponse>> {
    status::Custom(status, Json(models::ErrorResponse { message }))
}

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = diesel::sqlite::SqliteConnection::establish(&database_url)
        .expect("Could not connect to database");
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout())
        .expect("Could not apply database migrations");
    let active_session_ids: SessionStore = RwLock::new(HashSet::new());

    rocket::ignite()
        .attach(DBConnection::fairing())
        .mount("/", routes![routes::user::register])
        .register(catchers![server_error, not_found])
        .manage(active_session_ids)
        .launch();
}
