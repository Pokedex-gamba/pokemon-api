use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, LazyLock},
};

use crate::macros::{resp_404_NotFound, resp_500_InternalServerError};
use actix_web::HttpResponse;
use serde::de::DeserializeOwned;
use tokio::sync::{Mutex, OwnedMutexGuard};

pub enum ErrorAction {
    ReturnNotFound,
    ReturnInternalServerError,
}

impl ErrorAction {
    fn into_response(&self) -> HttpResponse {
        match self {
            ErrorAction::ReturnNotFound => resp_404_NotFound!(),
            ErrorAction::ReturnInternalServerError => resp_500_InternalServerError!(),
        }
    }
}

static CACHE: LazyLock<Cache> = LazyLock::new(|| Cache::default());

#[derive(Default)]
struct Cache {
    inner:
        Arc<std::sync::Mutex<HashMap<(String, TypeId), Arc<Mutex<Option<Box<dyn Any + Send>>>>>>>,
}

struct CacheEntry<T> {
    inner: OwnedMutexGuard<Option<Box<dyn Any + Send>>>,
    any_type: PhantomData<T>,
}

impl<T: Send + 'static> CacheEntry<T> {
    pub fn get(&self) -> Option<&T> {
        let data = (&*self.inner).as_ref()?;
        let data = data.downcast_ref::<T>().unwrap();
        Some(data)
    }

    pub fn set(&mut self, val: T) {
        *self.inner = Some(Box::new(val));
    }
}

impl Cache {
    fn get_value_guard<T: Send + 'static + Clone>(
        &self,
        key: String,
    ) -> Arc<Mutex<Option<Box<dyn Any + Send>>>> {
        let mut guard = self.inner.lock().unwrap();
        let hash_map = &mut *guard;
        let key = (key, TypeId::of::<T>());
        hash_map.entry(key).or_default().clone()
    }

    pub async fn entry<T: Send + 'static + Clone>(&self, key: String) -> CacheEntry<T> {
        let data_guard = self.get_value_guard::<T>(key);
        let data_guard = data_guard.lock_owned().await;
        CacheEntry {
            inner: data_guard,
            any_type: PhantomData,
        }
    }
}

pub async fn get_json<T: DeserializeOwned + 'static + Clone + Send>(
    req_client: &reqwest::Client,
    url: &str,
    on_request_error: ErrorAction,
    on_decode_error: ErrorAction,
) -> Result<T, HttpResponse> {
    let mut entry = CACHE.entry::<T>(url.to_string()).await;
    if let Some(data) = entry.get() {
        return Ok(data.clone());
    }

    let response = req_client.get(url).send().await;
    match response {
        Ok(res) => match res.json::<T>().await {
            Ok(data) => {
                entry.set(data.clone());
                Ok(data)
            }
            Err(_) => Err(on_decode_error.into_response()),
        },
        Err(_) => Err(on_request_error.into_response()),
    }
}
