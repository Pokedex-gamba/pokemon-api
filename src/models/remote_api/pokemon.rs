use serde::Deserialize;

use super::ApiPokemonSprites;

#[derive(Deserialize, Clone)]
pub struct ApiPokemon {
    pub name: String,
    #[serde(rename = "pokemon_v2_pokemonsprites")]
    pub sprites: Vec<ApiPokemonSprites>,
}
