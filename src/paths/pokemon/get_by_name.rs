use actix_web::{
    get,
    http::StatusCode,
    web::{self, Data},
    HttpResponse, Responder,
};
use serde_json::json;

use crate::{
    cache::CACHE,
    macros::{resp_200_Ok_json, yeet_error},
    models::{pokemon::Pokemon, remote_api::ApiPokemonList, DataWrapper},
    req_util::{self, response_from_error},
};

#[utoipa::path(
    responses(
        (status = 200, description = "Returns pokemon by name", body = Pokemon),
        (status = 400, description = "Parameter name has wrong type or isn't only alphabetic ASCII characters"),
        (status = 404, description = "Pokemon was not found"),
        (status = 500, description = "Failed to fetch/deserialize data from remote api"),
    ),
    security(
        ("jwt_grants" = ["svc::pokemon_api::route::/pokemon/get_by_name"]),
    )
)]
#[actix_web_grants::protect("svc::pokemon_api::route::/pokemon/get_by_name")]
#[get("/pokemon/get_by_name/{name}")]
pub async fn get_by_name(
    name: web::Path<String>,
    req_client: Data<reqwest::Client>,
) -> impl Responder {
    if name.chars().any(|c| !c.is_ascii_alphabetic()) {
        return response_from_error(
            "Name must be only ASCII alphabetic",
            StatusCode::BAD_REQUEST,
        );
    }

    let failed_to_convert = |_| {
        response_from_error(
            "Failed to convert api pokemon to our pokemon",
            StatusCode::NOT_FOUND,
        )
    };

    let entry = CACHE.entry(get_cache_key_for_pokemon(&name)).await;
    let mut lock = match entry.get_or_write_lock().await {
        actix_web::Either::Left(api_pokemon) => {
            let pokemon = Pokemon::try_from(&*api_pokemon).map_err(failed_to_convert);
            let pokemon = yeet_error!(pokemon);
            return resp_200_Ok_json!(pokemon);
        }
        actix_web::Either::Right(write_lock) => write_lock,
    };

    let res = req_util::post_json::<DataWrapper<ApiPokemonList>, HttpResponse>(
        &req_client,
        "https://beta.pokeapi.co/graphql/v1beta",
        &json!(
            {
                "query": crate::queries::GET_POKEMON.replacen("$name", &name, 1),
                "variables": null,
                "operationName": "GetPokemon"
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

    let mut api_pokemon = yeet_error!(res);
    if api_pokemon.data.results.len() != 1 {
        return response_from_error("Pokemon was not found", StatusCode::NOT_FOUND);
    };
    let api_pokemon = api_pokemon.data.results.remove(0);

    let pokemon = Pokemon::try_from(&api_pokemon)
        .map_err(failed_to_convert)
        .map(|pokemon| resp_200_Ok_json!(pokemon));

    lock.set(api_pokemon);
    yeet_error!(pokemon)
}

#[inline]
pub fn get_cache_key_for_pokemon(pokemon_name: &str) -> String {
    format!("pokemon//{pokemon_name}")
}
