use crate::schema::users;
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Deserialize, Queryable, Clone)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[table_name = "users"]
#[derive(Deserialize, Insertable)]
pub struct UserCreate {
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Serialize, Queryable)]
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