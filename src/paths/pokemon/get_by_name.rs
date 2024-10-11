use actix_web::{
    get,
    web::{self, Data},
    Responder,
};

use crate::{
    macros::{resp_200_Ok_json, yeet_error},
    models::{remote_api::ApiPokemon, Pokemon},
    req_caching::{self, ErrorAction},
};

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

    let api_pokemon = yeet_error!(res);

    let pokemon = Pokemon::from(api_pokemon);
    resp_200_Ok_json!(pokemon)
}
