use serde::Deserialize;

use super::ApiPokemonSprites;

#[derive(Deserialize)]
pub struct ApiPokemon {
    pub name: String,
    pub sprites: ApiPokemonSprites,
}
