use serde::Serialize;

use super::{
    remote_api::{ApiPokemon, ApiPokemonSpritesOtherOfficialArtwork},
    PokemonPictures,
};

#[derive(Serialize)]
pub struct Pokemon<'a> {
    pub name: &'a str,
    pub pictures: PokemonPictures<'a>,
}

impl<'a> TryFrom<&'a ApiPokemon> for Pokemon<'a> {
    type Error = ();

    fn try_from(value: &'a ApiPokemon) -> Result<Self, Self::Error> {
        let sprites = &value.sprites.other.official_artwork;
        if let ApiPokemonSpritesOtherOfficialArtwork {
            front_default: Some(front_default),
            front_shiny: Some(front_shiny),
        } = sprites
        {
            return Ok(Self {
                name: &value.name,
                pictures: PokemonPictures {
                    front_default,
                    front_shiny,
                },
            });
        }
        Err(())
    }
}
