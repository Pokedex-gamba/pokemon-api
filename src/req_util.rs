use std::fmt::Debug;

use crate::{empty_error::EmptyError, json_error::JsonError, IS_DEBUG_ON};
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::{de::DeserializeOwned, Serialize};

pub async fn handle_request<D: Serialize, T: DeserializeOwned + Send + Sync + 'static, E>(
    req_client: &reqwest::Client,
    method: reqwest::Method,
    url: &str,
    data: Option<&D>,
    on_error: impl Fn(reqwest::Error) -> E,
) -> Result<T, E> {
    let mut request = req_client.request(method, url);
    if let Some(data) = data {
        request = request.json(data);
    }
    let response = request.send().await;
    match response.and_then(reqwest::Response::error_for_status) {
        Ok(res) => match res.json::<T>().await {
            Ok(data) => Ok(data),
            Err(e) => Err((on_error)(e)),
        },
        Err(e) => Err((on_error)(e)),
    }
}

pub async fn post_json<T: DeserializeOwned + Send + Sync + 'static, E>(
    req_client: &reqwest::Client,
    url: &str,
    data: &impl Serialize,
    on_error: impl Fn(reqwest::Error) -> E,
) -> Result<T, E> {
    handle_request(req_client, reqwest::Method::POST, url, Some(data), on_error).await
}

pub fn response_from_error(error: impl Serialize + Debug, status_code: StatusCode) -> HttpResponse {
    if unsafe { IS_DEBUG_ON } {
        JsonError::new(error, status_code).error_response()
    } else {
        EmptyError::new(status_code).error_response()
    }
}
