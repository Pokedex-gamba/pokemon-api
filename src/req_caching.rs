use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, LazyLock},
};

use crate::{empty_error::EmptyError, json_error::JsonError, IS_DEBUG_ON};
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::{Mutex, OwnedMutexGuard};

pub static CACHE: LazyLock<Cache> = LazyLock::new(Cache::default);

#[derive(Default)]
pub struct Cache {
    #[allow(clippy::type_complexity)]
    inner:
        Arc<std::sync::Mutex<HashMap<(String, TypeId), Arc<Mutex<Option<Box<dyn Any + Send>>>>>>>,
}

pub struct CacheEntry<T> {
    inner: OwnedMutexGuard<Option<Box<dyn Any + Send>>>,
    any_type: PhantomData<T>,
}

impl<T: Send + 'static> CacheEntry<T> {
    pub fn get(&self) -> Option<&T> {
        let data = (*self.inner).as_ref()?;
        let data = data.downcast_ref::<T>().unwrap();
        Some(data)
    }

    pub fn set(&mut self, val: T) {
        *self.inner = Some(Box::new(val));
    }

    pub fn exists(&self) -> bool {
        (*self.inner).is_some()
    }
}

impl Cache {
    fn get_value_guard<T: 'static>(&self, key: String) -> Arc<Mutex<Option<Box<dyn Any + Send>>>> {
        let mut guard = self.inner.lock().unwrap();
        let hash_map = &mut *guard;
        let key = (key, TypeId::of::<T>());
        hash_map.entry(key).or_default().clone()
    }

    pub async fn entry<T: Send + 'static>(&self, key: String) -> CacheEntry<T> {
        let data_guard = self.get_value_guard::<T>(key);
        let data_guard = data_guard.lock_owned().await;
        CacheEntry {
            inner: data_guard,
            any_type: PhantomData,
        }
    }
}

pub async fn get_json<T: DeserializeOwned + 'static + Send, E>(
    req_client: &reqwest::Client,
    url: &str,
    on_error: impl Fn(reqwest::Error) -> E,
) -> Result<RefVal<T>, E> {
    let mut entry = CACHE.entry::<T>(url.to_string()).await;
    if entry.exists() {
        return Ok(RefVal(entry));
    }

    let response = req_client.get(url).send().await;
    match response.and_then(reqwest::Response::error_for_status) {
        Ok(res) => match res.json::<T>().await {
            Ok(data) => {
                entry.set(data);
                Ok(RefVal(entry))
            }
            Err(e) => Err((on_error)(e)),
        },
        Err(e) => Err((on_error)(e)),
    }
}

pub struct RefVal<T>(CacheEntry<T>);

impl<T: Send + 'static> Deref for RefVal<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.get().unwrap()
    }
}

pub fn response_from_error(error: impl Serialize + Debug, status_code: StatusCode) -> HttpResponse {
    if unsafe { IS_DEBUG_ON } {
        JsonError::new(error, status_code).error_response()
    } else {
        EmptyError::new(status_code).error_response()
    }
}
