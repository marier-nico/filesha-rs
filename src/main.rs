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
use std::collections::HashSet;
use std::env;

embed_migrations!();

mod api_error;
mod models;
mod passwords;
mod schema;
mod routes {
    pub mod user;
}

type SessionStore = RwLock<HashSet<i32>>;

#[database("data_db")]
pub struct DBConnection(SqliteConnection);

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
        .mount(
            "/user",
            routes![routes::user::register, routes::user::login],
        )
        .register(catchers![
            api_error::server_error,
            api_error::not_found,
            api_error::unprocessable_entity
        ])
        .manage(active_session_ids)
        .launch();
}
