use serde::Serialize;

#[derive(Serialize)]
pub struct PokemonPictures {
    pub front_default: String,
    pub front_shiny: String,
}
