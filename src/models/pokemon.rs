use serde::Serialize;
use utoipa::ToSchema;

use super::{
    pokemon_pictures::PokemonPictures,
    remote_api::{ApiPokemon, ApiPokemonSpritesOfficialArtwork},
};

#[derive(Serialize, ToSchema)]
pub struct Pokemon<'a> {
    pub name: &'a str,
    pub pictures: PokemonPictures<'a>,
}

impl<'a> TryFrom<&'a ApiPokemon> for Pokemon<'a> {
    type Error = ();

    fn try_from(value: &'a ApiPokemon) -> Result<Self, Self::Error> {
        if value.sprites.is_empty() {
            return Err(());
        }
        let sprites = &value.sprites[0].sprites;
        if let ApiPokemonSpritesOfficialArtwork {
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
