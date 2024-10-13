use actix_web::{
    get,
    web::{self, Data},
    Responder,
};

use crate::{
    macros::{resp_200_Ok_json, resp_404_NotFound_json, yeet_error},
    models::{pokemon::Pokemon, remote_api::ApiPokemon},
    req_caching::{self, ErrorAction},
};

#[utoipa::path(
    responses(
        (status = 200, description = "Returns pokemon by name", body = Pokemon),
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
    let res = req_caching::get_json::<ApiPokemon>(
        &**req_client,
        &format!("https://pokeapi.co/api/v2/pokemon/{}", name.into_inner()),
        ErrorAction::ReturnInternalServerError,
        ErrorAction::ReturnNotFound,
    )
    .await;

    let api_pokemon = &*yeet_error!(res);

    let pokemon = Pokemon::try_from(api_pokemon).map_err(|_| resp_404_NotFound_json!());
    let pokemon = yeet_error!(pokemon);
    resp_200_Ok_json!(pokemon)
}
