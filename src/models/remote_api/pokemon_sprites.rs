use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ApiPokemonSprites {
    pub other: ApiPokemonSpritesOther,
}

#[derive(Deserialize, Clone)]
pub struct ApiPokemonSpritesOther {
    #[serde(rename = "official-artwork")]
    pub official_artwork: ApiPokemonSpritesOtherOfficialArtwork,
}

#[derive(Deserialize, Clone)]
pub struct ApiPokemonSpritesOtherOfficialArtwork {
    pub front_default: String,
    pub front_shiny: String,
}
