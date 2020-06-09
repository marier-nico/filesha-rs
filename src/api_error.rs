use crate::passwords::PasswordError;
use diesel::result::Error as DieselError;
use rocket::http::Status;
use rocket::response;
use rocket::response::{status, Responder};
use rocket::Request;
use rocket_contrib::json::Json;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    InternalServerError,
    MissingEnvVars(Vec<String>),
    Custom(status::Custom<Json<ErrorResponse>>),
}

#[derive(Debug)]
pub struct CustomError {
    pub message: String,
    pub status: Status,
}

impl CustomError {
    pub fn new(msg: String, status: Status) -> Self {
        CustomError {
            message: msg,
            status,
        }
    }
}

#[derive(serde::Serialize, Debug)]
pub struct ErrorResponse {
    message: String,
}

impl<'r> Responder<'r> for ApiError {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        match self {
            ApiError::NotFound => Err(Status::NotFound),
            ApiError::Custom(error) => error.respond_to(request),
            _ => Err(Status::InternalServerError),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            ApiError::NotFound => f.write_str("NotFound"),
            ApiError::InternalServerError => f.write_str("InternalServerError"),
            ApiError::MissingEnvVars(vars) => f.write_str(&format!("Missing env vars: {:#?}", vars)),
            ApiError::Custom(_) => f.write_str("CustomJsonError"),
        }
    }
}

impl StdError for ApiError {
    fn description(&self) -> &str {
        match *self {
            ApiError::NotFound => "Record not found",
            ApiError::InternalServerError => "Internal server error",
            ApiError::MissingEnvVars(_) => "Missing environment variables",
            ApiError::Custom(_) => "Custom JSON error message",
        }
    }
}

impl From<DieselError> for ApiError {
    fn from(e: DieselError) -> Self {
        match e {
            DieselError::NotFound => ApiError::NotFound,
            _ => ApiError::InternalServerError,
        }
    }
}

impl From<PasswordError> for ApiError {
    fn from(_e: PasswordError) -> Self {
        ApiError::InternalServerError
    }
}

impl From<std::io::Error> for ApiError {
    fn from(_: std::io::Error) -> Self {
        ApiError::InternalServerError
    }
}

impl From<CustomError> for ApiError {
    fn from(e: CustomError) -> Self {
        ApiError::Custom(status::Custom(
            e.status,
            Json(ErrorResponse {
                message: e.message.to_string(),
            }),
        ))
    }
}

#[catch(401)]
pub fn unauthorized(_req: &rocket::Request) -> ApiError {
    ApiError::from(CustomError {
        status: Status::Unauthorized,
        message: "You may not access this resource".to_string(),
    })
}

#[catch(404)]
pub fn not_found(_req: &rocket::Request) -> ApiError {
    ApiError::from(CustomError {
        status: Status::NotFound,
        message: "Not found".to_string(),
    })
}

#[catch(422)]
pub fn unprocessable_entity(_req: &rocket::Request) -> ApiError {
    ApiError::from(CustomError {
        status: Status::UnprocessableEntity,
        message: "Invalid data format, please follow the API spec".to_string(),
    })
}

#[catch(500)]
pub fn server_error(_req: &rocket::Request) -> ApiError {
    ApiError::from(CustomError {
        status: Status::InternalServerError,
        message: "The server encountered an error processing your request".to_string(),
    })
}
