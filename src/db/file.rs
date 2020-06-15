use crate::api_error::{ApiError, CustomError};
use crate::models::file::Share;
use crate::schema::shares::table as shares_table;
use crate::schema::shares::link as link_column;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::SqliteConnection;
use rocket::http::Status;

pub fn get_share(link: &str, conn: &SqliteConnection) -> Result<Option<Share>, ApiError> {
    let result = shares_table
        .filter(link_column.eq(link))
        .limit(1)
        .load::<Share>(conn)?
        .into_iter()
        .next();

    Ok(result)
}

pub fn save_share(share: &Share, conn: &SqliteConnection) -> Result<(), ApiError> {
    insert_into(shares_table)
        .values(share)
        .execute(conn)
        .map_err(|e| CustomError::new(e.to_string(), Status::InternalServerError))?;
    
    Ok(())
}