use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiPokemonSprites {
    pub other: ApiPokemonSpritesOther,
}

#[derive(Deserialize)]
pub struct ApiPokemonSpritesOther {
    #[serde(rename = "official-artwork")]
    pub official_artwork: ApiPokemonSpritesOtherOfficialArtwork,
}

#[derive(Deserialize)]
pub struct ApiPokemonSpritesOtherOfficialArtwork {
    pub front_default: String,
    pub front_shiny: String,
}
