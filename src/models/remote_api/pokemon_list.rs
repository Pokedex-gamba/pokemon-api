use serde::Deserialize;

use super::ApiPokemon;

#[derive(Deserialize)]
pub struct ApiPokemonList {
    #[serde(rename = "pokemon_v2_pokemon")]
    pub results: Vec<ApiPokemon>,
}
