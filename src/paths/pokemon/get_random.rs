use std::num::NonZeroU8;

use actix_web::{
    get,
    web::{self, Data},
    Responder,
};
use rand::Rng;

use crate::{
    macros::{resp_200_Ok_json, yeet_error},
    models::{
        remote_api::{ApiPokemon, ApiPokemonList},
        Pokemon,
    },
    req_caching::{self, ErrorAction},
};

#[get("/pokemon/get_random/{count}")]
pub async fn get_random(
    count: web::Path<NonZeroU8>,
    req_client: Data<reqwest::Client>,
) -> impl Responder {
    let res = req_caching::get_json::<ApiPokemonList>(
        &**req_client,
        "https://pokeapi.co/api/v2/pokemon?limit=99999",
        ErrorAction::ReturnInternalServerError,
        ErrorAction::ReturnInternalServerError,
    )
    .await;

    let pokemon_list = &*yeet_error!(res).results;
    let mut pokemons = Vec::with_capacity(count.get() as usize);
    let mut rng = rand::thread_rng();

    loop {
        let i = rng.gen_range(0..pokemon_list.len());
        let res = req_caching::get_json::<ApiPokemon>(
            &**req_client,
            &format!("https://pokeapi.co/api/v2/pokemon/{}", pokemon_list[i].name),
            ErrorAction::ReturnNotFound,
            ErrorAction::ReturnInternalServerError,
        )
        .await;

        match res {
            Ok(res) if Pokemon::try_from(&*res).is_ok() => pokemons.push(res.name.clone()),
            _ => {}
        }

        if pokemons.len() == count.get() as usize {
            break;
        }
    }

    resp_200_Ok_json!(pokemons)
}
