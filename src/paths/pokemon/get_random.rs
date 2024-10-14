use actix_web::{
    get,
    web::{self, Data},
    Responder,
};
use rand::Rng;

use crate::{
    macros::{resp_200_Ok_json, yeet_error},
    models::{
        pokemon::Pokemon,
        remote_api::{ApiPokemon, ApiPokemonList},
    },
    req_caching::{self, ErrorAction},
};

#[utoipa::path(
    responses(
        (status = 200, description = "Returns N random pokemons", body = [Pokemon]),
    ),
    security(
        ("jwt_grants" = ["svc::pokemon_api::route::/pokemon/get_random"]),
    )
)]
#[actix_web_grants::protect("svc::pokemon_api::route::/pokemon/get_random")]
#[get("/pokemon/get_random/{count}")]
pub async fn get_random(count: web::Path<u8>, req_client: Data<reqwest::Client>) -> impl Responder {
    let res = req_caching::get_json::<ApiPokemonList>(
        &req_client,
        "https://pokeapi.co/api/v2/pokemon?limit=99999",
        ErrorAction::ReturnInternalServerError,
        ErrorAction::ReturnInternalServerError,
    )
    .await;

    let pokemon_list = &*yeet_error!(res).results;
    let mut pokemons = Vec::with_capacity(*count as usize);
    let mut rng = rand::thread_rng();

    loop {
        if pokemons.len() == *count as usize {
            break;
        }

        let i = rng.gen_range(0..pokemon_list.len());
        let res = req_caching::get_json::<ApiPokemon>(
            &req_client,
            &format!("https://pokeapi.co/api/v2/pokemon/{}", pokemon_list[i].name),
            ErrorAction::ReturnNotFound,
            ErrorAction::ReturnInternalServerError,
        )
        .await;

        match res {
            Ok(res) if Pokemon::try_from(&*res).is_ok() => pokemons.push(res.name.clone()),
            _ => {}
        }
    }

    resp_200_Ok_json!(pokemons)
}
