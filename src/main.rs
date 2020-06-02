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
use parking_lot::RwLock;
use rocket_contrib::databases::diesel::SqliteConnection;
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

embed_migrations!();

mod api_error;
mod guards;
mod models;
mod passwords;
mod schema;
mod routes {
    pub mod user;
    pub mod file;
}

type Email = String;
type SessionStore = RwLock<HashMap<Uuid, Email>>;
type PendingUploadStore = RwLock<HashMap<Uuid, models::PendingUpload>>;

#[database("data_db")]
pub struct DBConnection(SqliteConnection);

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = diesel::sqlite::SqliteConnection::establish(&database_url)
        .expect("Could not connect to database");
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout())
        .expect("Could not apply database migrations");
    let active_session_ids: SessionStore = RwLock::new(HashMap::new());
    let pending_uploads: PendingUploadStore = RwLock::new(HashMap::new());

    rocket::ignite()
        .attach(DBConnection::fairing())
        .mount(
            "/user",
            routes![routes::user::register, routes::user::login],
        )
        .mount("/file", routes![routes::file::new_upload])
        .register(catchers![
            api_error::server_error,
            api_error::not_found,
            api_error::unprocessable_entity
        ])
        .manage(active_session_ids)
        .manage(pending_uploads)
        .launch();
}
