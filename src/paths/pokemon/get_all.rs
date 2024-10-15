use actix_web::{get, http::StatusCode, web::Data, HttpResponse, Responder};
use futures::stream::StreamExt;

use crate::{
    macros::{resp_200_Ok_json, yeet_error},
    models::{
        pokemon::Pokemon,
        remote_api::{ApiPokemon, ApiPokemonList},
    },
    req_caching::{self, response_from_error, CACHE},
};

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
    let mut entry = CACHE.entry::<String>("get_all route".into()).await;
    if let Some(data) = entry.get() {
        return resp_200_Ok_json!(data.clone(), raw);
    }

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

    let pokemon_list = &*yeet_error!(res);

    let api_pokemons = futures::stream::iter(pokemon_list.results.iter())
        .map(|pokemon| async {
            req_caching::get_json::<ApiPokemon, ()>(
                &req_client,
                &format!("https://pokeapi.co/api/v2/pokemon/{}", pokemon.name),
                |_| (),
            )
            .await
            .ok()
        })
        .buffered(50)
        .filter_map(|res| async { res })
        .collect::<Vec<_>>()
        .await;

    let pokemons = api_pokemons
        .iter()
        .filter_map(|api_pokemon| Pokemon::try_from(&**api_pokemon).ok())
        .collect::<Vec<_>>();

    let data = serde_json::to_string(&pokemons).unwrap();
    entry.set(data.clone());
    resp_200_Ok_json!(data, raw)
}
