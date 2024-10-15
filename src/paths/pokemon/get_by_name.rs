use actix_web::{
    get,
    http::StatusCode,
    web::{self, Data},
    HttpResponse, Responder,
};

use crate::{
    macros::{resp_200_Ok_json, resp_404_NotFound_json, yeet_error},
    models::{pokemon::Pokemon, remote_api::ApiPokemon},
    req_caching::{self, response_from_error},
};

#[utoipa::path(
    responses(
        (status = 200, description = "Returns pokemon by name", body = Pokemon),
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
    let res = req_caching::get_json::<ApiPokemon, HttpResponse>(
        &req_client,
        &format!("https://pokeapi.co/api/v2/pokemon/{}", name.into_inner()),
        |error| {
            let (error, status_code) = handle_error(error);
            response_from_error(error, status_code)
        },
    )
    .await;

    let api_pokemon = &*yeet_error!(res);

    let pokemon = Pokemon::try_from(api_pokemon).map_err(|_| resp_404_NotFound_json!());
    let pokemon = yeet_error!(pokemon);
    resp_200_Ok_json!(pokemon)
}

fn handle_error(error: reqwest::Error) -> (String, StatusCode) {
    match error.status() {
        Some(reqwest::StatusCode::NOT_FOUND) => {
            ("Pokemon was not found".into(), StatusCode::NOT_FOUND)
        }
        _ => (
            format!("Error encountered: {error}"),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}
