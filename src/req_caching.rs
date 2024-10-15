use actix_web::Either;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    cache::{RefVal, CACHE},
    req_util::handle_request,
};

pub async fn handle_cache_request<D: Serialize, T: DeserializeOwned + Send + Sync + 'static, E>(
    req_client: &reqwest::Client,
    method: reqwest::Method,
    cache_key: String,
    url: &str,
    data: Option<&D>,
    on_error: impl Fn(reqwest::Error) -> E,
) -> Result<RefVal<T>, E> {
    let entry = CACHE.entry::<T>(cache_key).await;
    let mut data_lock = match entry.get_or_write_lock().await {
        Either::Left(data) => return Ok(data),
        Either::Right(write_lock) => write_lock,
    };

    let data = handle_request(req_client, method, url, data, on_error).await?;
    data_lock.set(data);
    drop(data_lock);
    Ok(RefVal(entry.read().await))
}

#[allow(dead_code)]
pub async fn get_json_cached<T: DeserializeOwned + Send + Sync + 'static, E>(
    req_client: &reqwest::Client,
    url: &str,
    on_error: impl Fn(reqwest::Error) -> E,
) -> Result<RefVal<T>, E> {
    handle_cache_request(
        req_client,
        reqwest::Method::POST,
        url.to_string(),
        url,
        Option::<&()>::None,
        on_error,
    )
    .await
}

pub async fn post_json_cached<T: DeserializeOwned + Send + Sync + 'static, E>(
    req_client: &reqwest::Client,
    cache_key: impl Into<String>,
    url: &str,
    data: &impl Serialize,
    on_error: impl Fn(reqwest::Error) -> E,
) -> Result<RefVal<T>, E> {
    handle_cache_request(
        req_client,
        reqwest::Method::POST,
        cache_key.into(),
        url,
        Some(data),
        on_error,
    )
    .await
}
