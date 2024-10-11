use serde::Serialize;

use super::{remote_api::ApiPokemon, PokemonPictures};

#[derive(Serialize)]
pub struct Pokemon {
    pub name: String,
    pub pictures: PokemonPictures,
}

impl From<ApiPokemon> for Pokemon {
    fn from(value: ApiPokemon) -> Self {
        Self {
            name: value.name,
            pictures: PokemonPictures {
                front_default: value.sprites.other.official_artwork.front_default,
                front_shiny: value.sprites.other.official_artwork.front_shiny,
            },
        }
    }
}
