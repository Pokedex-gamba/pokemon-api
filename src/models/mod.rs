pub mod pokemon;
pub mod pokemon_pictures;
pub mod remote_api;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct DataWrapper<T> {
    pub data: T,
}
