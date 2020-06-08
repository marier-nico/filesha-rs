use crate::api_error::{ApiError, CustomError};
use crate::models::user::{User, UserCreate};
use crate::schema::users::email as email_column;
use crate::schema::users::table as users_table;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::SqliteConnection;
use rocket::http::Status;

pub fn create(user: &UserCreate, conn: &SqliteConnection) -> Result<(), ApiError> {
    insert_into(users_table)
        .values(user)
        .execute(conn)
        .map_err(|_| CustomError::new("This user is already registered", Status::BadRequest))?;

    Ok(())
}

pub fn get_by_email(email: &str, conn: &SqliteConnection) -> Result<Option<User>, ApiError> {
    let result = users_table
        .filter(email_column.eq(email))
        .limit(1)
        .load::<User>(conn)?
        .into_iter()
        .next();

    Ok(result)
}
