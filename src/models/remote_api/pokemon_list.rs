use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiPokemonList {
    pub results: Vec<ApiPokemonListItem>,
}

#[derive(Deserialize)]
pub struct ApiPokemonListItem {
    pub name: String,
}
