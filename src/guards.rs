use crate::models::user::User;
use crate::schema::users;
use crate::DBConnection;
use crate::SessionStore;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use rocket::State;
use uuid::Uuid;

#[derive(Debug)]
pub enum AuthenticationError {
    Unauthenticated,
    ServerError
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = AuthenticationError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let cookie = match request.cookies().get_private("session") {
            Some(cookie) => cookie,
            None => {
                return Outcome::Failure((
                    Status::Unauthorized,
                    AuthenticationError::Unauthenticated,
                ))
            }
        };
        let cookie_value = cookie.value();
        let session_id = match Uuid::parse_str(cookie_value) {
            Ok(uuid) => uuid,
            Err(_) => {
                return Outcome::Failure((
                    Status::Unauthorized,
                    AuthenticationError::Unauthenticated,
                ))
            }
        };

        let session_store = match request.guard::<State<SessionStore>>() {
            Outcome::Success(session_store) => session_store,
            _ => {
                return Outcome::Failure((
                    Status::InternalServerError,
                    AuthenticationError::ServerError,
                ))
            }
        };
        let active_session_ids = session_store.read();
        let user_email = match active_session_ids.get(&session_id) {
            Some(email) => email,
            None => {
                return Outcome::Failure((
                    Status::Unauthorized,
                    AuthenticationError::Unauthenticated,
                ))
            }
        };

        let conn = match request.guard::<DBConnection>() {
            Outcome::Success(conn) => conn,
            _ => {
                return Outcome::Failure((
                    Status::InternalServerError,
                    AuthenticationError::ServerError,
                ))
            }
        };
        let query_result = match users::table
            .filter(users::email.eq(user_email))
            .limit(1)
            .load::<User>(&*conn)
        {
            Ok(result) => result,
            Err(_) => {
                return Outcome::Failure((
                    Status::InternalServerError,
                    AuthenticationError::ServerError,
                ))
            }
        };
        let db_user = match query_result.into_iter().next() {
            Some(user) => user,
            None => {
                return Outcome::Failure((
                    Status::Unauthorized,
                    AuthenticationError::Unauthenticated,
                ))
            }
        };

        Outcome::Success(db_user)
    }
}
