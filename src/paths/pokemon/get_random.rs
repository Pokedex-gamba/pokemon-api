use actix_web::{
    get,
    http::StatusCode,
    web::{self, Data},
    HttpResponse, Responder,
};
use rand::Rng;
use serde_json::json;

use super::get_all;
use crate::{
    macros::{resp_200_Ok_json, yeet_error},
    models::{pokemon::Pokemon, remote_api::ApiPokemonList, DataWrapper},
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
    let res = req_caching::post_json_cached::<DataWrapper<ApiPokemonList>, HttpResponse>(
        &req_client,
        get_all::CACHE_KEY,
        "https://beta.pokeapi.co/graphql/v1beta",
        &json!(
            {
                "query": crate::queries::GET_ALL_POKEMONS,
                "variables": null,
                "operationName": "GetAllPokemons"
            }
        ),
        |error| {
            response_from_error(
                format!("Error encountered: {error}"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        },
    )
    .await;

    let pokemon_list = &yeet_error!(res).data.results;
    let mut pokemons = Vec::with_capacity(*count as usize);
    let mut rng = rand::thread_rng();

    loop {
        if pokemons.len() == *count as usize {
            break;
        }

        let i = rng.gen_range(0..pokemon_list.len());
        if let Ok(pokemon) = Pokemon::try_from(&pokemon_list[i]) {
            pokemons.push(pokemon);
        }
    }

    resp_200_Ok_json!(pokemons)
}
