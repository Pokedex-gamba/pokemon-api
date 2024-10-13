use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct PokemonPictures<'a> {
    pub front_default: &'a str,
    pub front_shiny: &'a str,
}
