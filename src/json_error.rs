use std::fmt::{Debug, Display};

use actix_web::{
    body::BoxBody, http::StatusCode, HttpRequest, HttpResponse, HttpResponseBuilder, ResponseError,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct JsonError<Err> {
    error: Err,
    #[serde(skip)]
    status_code: StatusCode,
}

impl<Err> JsonError<Err> {
    pub fn new(error: Err, status_code: StatusCode) -> Self {
        Self { error, status_code }
    }
}

impl<Err: Debug> Display for JsonError<Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonError")
            .field("error", &self.error)
            .field("status_code", &self.status_code)
            .finish()
    }
}

impl From<&dyn actix_web::ResponseError> for JsonError<String> {
    fn from(value: &dyn actix_web::ResponseError) -> Self {
        Self {
            status_code: value.status_code(),
            error: value.to_string(),
        }
    }
}

impl<Err: Serialize + Debug> ResponseError for JsonError<Err> {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponseBuilder::new(self.status_code).json(self)
    }
}

pub fn json_config_error_handler<Err: actix_web::ResponseError + 'static>(
    err: Err,
    _: &HttpRequest,
) -> actix_web::Error {
    JsonError::from(&err as &dyn actix_web::ResponseError).into()
}
