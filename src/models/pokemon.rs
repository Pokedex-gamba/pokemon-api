use serde::Serialize;

use super::{remote_api::ApiPokemon, PokemonPictures};

#[derive(Serialize)]
pub struct Pokemon<'a> {
    pub name: &'a str,
    pub pictures: PokemonPictures<'a>,
}

impl<'a> From<&'a ApiPokemon> for Pokemon<'a> {
    fn from(value: &'a ApiPokemon) -> Self {
        Self {
            name: &value.name,
            pictures: PokemonPictures {
                front_default: &value.sprites.other.official_artwork.front_default,
                front_shiny: &value.sprites.other.official_artwork.front_shiny,
            },
        }
    }
}
