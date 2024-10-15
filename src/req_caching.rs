use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, LazyLock},
};

use crate::{empty_error::EmptyError, json_error::JsonError, IS_DEBUG_ON};
use actix_web::{http::StatusCode, Either, HttpResponse, ResponseError};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

pub static CACHE: LazyLock<Cache> = LazyLock::new(Cache::default);

#[derive(Default)]
pub struct Cache {
    #[allow(clippy::type_complexity)]
    inner: Arc<
        std::sync::RwLock<
            HashMap<(String, TypeId), Arc<RwLock<Option<Box<dyn Any + Send + Sync>>>>>,
        >,
    >,
}

pub struct CacheEntry<T> {
    inner: Arc<RwLock<Option<Box<dyn Any + Send + Sync>>>>,
    any_type: PhantomData<T>,
}

pub struct ReadCacheEntryValue<T> {
    inner: OwnedRwLockReadGuard<Option<Box<dyn Any + Send + Sync>>>,
    any_type: PhantomData<T>,
}

impl<T: 'static> ReadCacheEntryValue<T> {
    pub fn get(&self) -> Option<&T> {
        let data = (*self.inner).as_ref()?;
        let data = data.downcast_ref::<T>().unwrap();
        Some(data)
    }
}

#[allow(dead_code)]
impl<T: Send + Sync + 'static> WriteCacheEntryValue<T> {
    pub fn get(&self) -> Option<&T> {
        let data = (*self.inner).as_ref()?;
        let data = data.downcast_ref::<T>().unwrap();
        Some(data)
    }

    pub fn set(&mut self, val: T) {
        *self.inner = Some(Box::new(val));
    }
}

pub struct WriteCacheEntryValue<T> {
    inner: OwnedRwLockWriteGuard<Option<Box<dyn Any + Send + Sync>>>,
    any_type: PhantomData<T>,
}

#[allow(dead_code)]
impl<T: Send + Sync + 'static> CacheEntry<T> {
    pub async fn read(&self) -> ReadCacheEntryValue<T> {
        let data = self.inner.clone().read_owned().await;
        ReadCacheEntryValue {
            inner: data,
            any_type: PhantomData,
        }
    }

    pub async fn write(&self) -> WriteCacheEntryValue<T> {
        let data = self.inner.clone().write_owned().await;
        WriteCacheEntryValue {
            inner: data,
            any_type: PhantomData,
        }
    }

    pub async fn get_or_write_lock(&self) -> Either<RefVal<T>, WriteCacheEntryValue<T>> {
        loop {
            let read_guard = self.inner.read().await;
            if read_guard.is_some() {
                return Either::Left(RefVal(self.read().await));
            }
            drop(read_guard);
            let write_guard = self.inner.clone().write_owned().await;
            if write_guard.is_some() {
                continue;
            }
            return Either::Right(WriteCacheEntryValue {
                inner: write_guard,
                any_type: PhantomData,
            });
        }
    }
}

impl Cache {
    fn get_value_guard<T: 'static>(
        &self,
        key: String,
    ) -> Arc<RwLock<Option<Box<dyn Any + Send + Sync>>>> {
        let key = (key, TypeId::of::<T>());
        let hash_map = self.inner.read().unwrap();
        if let Some(data) = hash_map.get(&key) {
            return data.clone();
        }
        drop(hash_map);
        let mut hash_map = self.inner.write().unwrap();
        hash_map.entry(key).or_default().clone()
    }

    pub async fn entry<T: Send + 'static>(&self, key: String) -> CacheEntry<T> {
        let data_guard = self.get_value_guard::<T>(key);
        CacheEntry {
            inner: data_guard,
            any_type: PhantomData,
        }
    }
}

pub async fn get_json<T: DeserializeOwned + Send + Sync + 'static, E>(
    req_client: &reqwest::Client,
    url: &str,
    on_error: impl Fn(reqwest::Error) -> E,
) -> Result<RefVal<T>, E> {
    let entry = CACHE.entry::<T>(url.to_string()).await;
    let mut data_lock = match entry.get_or_write_lock().await {
        Either::Left(data) => return Ok(data),
        Either::Right(write_lock) => write_lock,
    };

    let response = req_client.get(url).send().await;
    match response.and_then(reqwest::Response::error_for_status) {
        Ok(res) => match res.json::<T>().await {
            Ok(data) => {
                data_lock.set(data);
                drop(data_lock);
                Ok(RefVal(entry.read().await))
            }
            Err(e) => Err((on_error)(e)),
        },
        Err(e) => Err((on_error)(e)),
    }
}

pub struct RefVal<T>(ReadCacheEntryValue<T>);

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
