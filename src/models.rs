use crate::schema::users;
use diesel::{Insertable, Queryable};
use serde;

#[derive(serde::Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[table_name = "users"]
#[derive(serde::Deserialize, Insertable)]
pub struct UserCreate {
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(serde::Serialize, Queryable)]
pub struct UserResult {
    pub id: i32,
    pub email: String,
    pub display_name: String,
}

impl UserResult {
    pub fn from(user: &User) -> UserResult {
        UserResult {
            id: user.id,
            email: user.email.to_string(),
            display_name: user.display_name.to_string(),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize, Debug)]
pub struct ErrorResponse {
    pub message: String,
}
