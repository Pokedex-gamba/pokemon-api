use serde::Deserialize;

use super::ApiPokemonSprites;

#[derive(Deserialize, Clone)]
pub struct ApiPokemon {
    pub name: String,
    pub sprites: ApiPokemonSprites,
}
