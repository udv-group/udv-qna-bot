use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;
use sqlx::Error;

use std::io::Cursor;

#[derive(Debug)]
pub enum ApiError {
    DataBaseError(sqlx::Error),
}

pub type EmptyResult = Result<(), ApiError>;
pub type JsonResult<T> = Result<Json<T>, ApiError>;

// TODO: return body on error
impl<'r> Responder<'r, 'r> for ApiError {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'r> {
        let (message, status) = match self {
            ApiError::DataBaseError(error) => match error {
                Error::RowNotFound => ("Object not found".to_owned(), Status::BadRequest),
                _ => {
                    let str_err = error.to_string();
                    dbg!(error);
                    (str_err, Status::InternalServerError)
                }
            },
        };
        Ok(Response::build()
            .status(status)
            .sized_body(message.len(), Cursor::new(message))
            .finalize())
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> ApiError {
        ApiError::DataBaseError(error)
    }
}
