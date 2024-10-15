use actix_web::{
    get,
    web::{self, Data},
    Responder,
};
use rand::Rng;

use super::get_all;
use crate::{
    macros::{resp_200_Ok_json, yeet_error},
    models::pokemon::Pokemon,
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
    let res = get_all::get_all_pokemons(&req_client).await;

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
