use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ApiPokemonSprites {
    pub sprites: ApiPokemonSpritesOfficialArtwork,
}

#[derive(Deserialize, Clone)]
pub struct ApiPokemonSpritesOfficialArtwork {
    pub front_default: Option<String>,
    pub front_shiny: Option<String>,
}
