use crate::schema::users;
use diesel::{Insertable, Queryable};
use serde;

#[derive(serde::Deserialize, Queryable, Insertable)]
pub struct User {
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(serde::Serialize, Queryable)]
pub struct UserResult {
    pub email: String,
    pub display_name: String,
}

#[derive(serde::Serialize, Debug)]
pub struct ErrorResponse {
    pub message: String
}
