use actix_web::{
    get,
    http::StatusCode,
    web::{self, Data},
    HttpResponse, Responder,
};
use rand::Rng;

use crate::{
    macros::{resp_200_Ok_json, yeet_error},
    models::{
        pokemon::Pokemon,
        remote_api::{ApiPokemon, ApiPokemonList},
    },
    req_caching::{self, response_from_error},
};

#[utoipa::path(
    responses(
        (status = 200, description = "Returns N random pokemons", body = [Pokemon]),
        (status = 500, description = "Failed to fetch/deserialize data from remote api"),
    ),
    security(
        ("jwt_grants" = ["svc::pokemon_api::route::/pokemon/get_random"]),
    )
)]
#[actix_web_grants::protect("svc::pokemon_api::route::/pokemon/get_random")]
#[get("/pokemon/get_random/{count}")]
pub async fn get_random(count: web::Path<u8>, req_client: Data<reqwest::Client>) -> impl Responder {
    let res = req_caching::get_json::<ApiPokemonList, HttpResponse>(
        &req_client,
        "https://pokeapi.co/api/v2/pokemon?limit=99999",
        |error| {
            response_from_error(
                format!("Error encountered: {error}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        },
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
        let res = req_caching::get_json::<ApiPokemon, ()>(
            &req_client,
            &format!("https://pokeapi.co/api/v2/pokemon/{}", pokemon_list[i].name),
            |_| (),
        )
        .await;

        match res {
            Ok(res) if Pokemon::try_from(&*res).is_ok() => pokemons.push(res.name.clone()),
            _ => {}
        }
    }

    resp_200_Ok_json!(pokemons)
}
