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
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

mod api_error;
mod guards;
mod passwords;
mod schema;
mod utils;
mod db {
    pub mod file;
    pub mod user;
}
mod models {
    pub mod common_models;
    pub mod file;
    pub mod user;
}
mod routes {
    pub mod file;
    pub mod user;
}

embed_migrations!();

type SessionStore = Arc<RwLock<HashMap<Uuid, models::user::ActiveSession>>>;
type PendingUploadStore = Arc<RwLock<HashMap<Uuid, models::file::PendingUpload>>>;

#[database("data_db")]
pub struct DBConnection(SqliteConnection);

fn main() {
    dotenv().ok();
    utils::ensure_all_env_vars_are_set().expect("Some required environment variables are not set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = diesel::sqlite::SqliteConnection::establish(&database_url)
        .expect("Could not connect to database");
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout())
        .expect("Could not apply database migrations");
    let active_session_ids: SessionStore = Arc::new(RwLock::new(HashMap::new()));
    let pending_uploads: PendingUploadStore = Arc::new(RwLock::new(HashMap::new()));

    let pending_uploads_thread = pending_uploads.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(60 * 60)); // Run the cleanup every hour
        let new_upload_store = utils::remove_old_pending_uploads(&*pending_uploads_thread.read());
        *pending_uploads_thread.write() = new_upload_store;
    });

    let active_session_ids_thread = active_session_ids.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(60 * 60)); // Run the cleanup every hour
        let new_active_sessions = utils::remove_old_sessions(&*active_session_ids_thread.read());
        *active_session_ids_thread.write() = new_active_sessions;
    });

    rocket::ignite()
        .attach(DBConnection::fairing())
        .mount(
            "/user",
            routes![routes::user::register, routes::user::login],
        )
        .mount(
            "/file",
            routes![
                routes::file::new_upload,
                routes::file::upload,
                routes::file::ls,
                routes::file::mkdir,
                routes::file::download,
                routes::file::create_share,
                routes::file::download_shared
            ],
        )
        .register(catchers![
            api_error::unauthorized,
            api_error::not_found,
            api_error::unprocessable_entity,
            api_error::server_error,
        ])
        .manage(active_session_ids)
        .manage(pending_uploads)
        .launch();
}
