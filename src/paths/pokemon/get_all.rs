use actix_web::{get, http::StatusCode, web::Data, HttpResponse, Responder};
use serde_json::json;

use crate::{
    macros::{resp_200_Ok_json, yeet_error},
    models::{pokemon::Pokemon, remote_api::ApiPokemonList, DataWrapper},
    req_caching,
    req_util::response_from_error,
};

pub const CACHE_KEY: &str = "/pokemon/get_all";

#[utoipa::path(
    responses(
        (status = 200, description = "Returns all pokemons", body = [Pokemon]),
        (status = 500, description = "Failed to fetch/deserialize data from remote api"),
    ),
    security(
        ("jwt_grants" = ["svc::pokemon_api::route::/pokemon/get_all"]),
    )
)]
#[actix_web_grants::protect("svc::pokemon_api::route::/pokemon/get_all")]
#[get("/pokemon/get_all")]
pub async fn get_all(req_client: Data<reqwest::Client>) -> impl Responder {
    let res = req_caching::post_json_cached::<DataWrapper<ApiPokemonList>, HttpResponse>(
        &req_client,
        CACHE_KEY,
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

    let pokemons = pokemon_list
        .iter()
        .filter_map(|api_pokemon| Pokemon::try_from(api_pokemon).ok())
        .collect::<Vec<_>>();

    resp_200_Ok_json!(pokemons)
}
