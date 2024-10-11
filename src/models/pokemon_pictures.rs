use serde::Serialize;

#[derive(Serialize)]
pub struct PokemonPictures<'a> {
    pub front_default: &'a str,
    pub front_shiny: &'a str,
}
