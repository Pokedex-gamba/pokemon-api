use std::fmt::{Debug, Display};

use actix_web::{
    body::BoxBody, http::StatusCode, HttpRequest, HttpResponse, HttpResponseBuilder, ResponseError,
};

#[derive(Debug)]
pub struct EmptyError {
    status_code: StatusCode,
}

impl EmptyError {
    pub fn new(status_code: StatusCode) -> Self {
        Self { status_code }
    }
}

impl Display for EmptyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonError")
            .field("status_code", &self.status_code)
            .finish()
    }
}

impl From<&dyn actix_web::ResponseError> for EmptyError {
    fn from(value: &dyn actix_web::ResponseError) -> Self {
        Self {
            status_code: value.status_code(),
        }
    }
}

impl ResponseError for EmptyError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponseBuilder::new(self.status_code).finish()
    }
}

pub fn config_empty_error_handler<Err: actix_web::ResponseError + 'static>(
    err: Err,
    _: &HttpRequest,
) -> actix_web::Error {
    EmptyError::from(&err as &dyn actix_web::ResponseError).into()
}
