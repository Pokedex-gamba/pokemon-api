use actix_web::{
    middleware::{Compress, Logger, NormalizePath, TrailingSlash},
    web::{self, Data, JsonConfig, PathConfig},
    App, HttpServer,
};
use actix_web_grants::GrantErrorConfig;
use jwt_stuff::JwtGrantsMiddleware;

mod empty_error;
mod json_error;
mod jwt_stuff;
mod macros;
mod models;
mod paths;
mod req_caching;

async fn default_handler_debug(req: actix_web::HttpRequest) -> impl actix_web::Responder {
    actix_web::HttpResponse::NotFound().body(format!("{:#?}", req))
}
async fn default_handler() -> impl actix_web::Responder {
    actix_web::HttpResponse::NotFound().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let is_debug_on = std::env::var("debug")
        .map(|val| val == "1")
        .unwrap_or_default();
    tracing::info!(
        "Debug is {}",
        if is_debug_on { "enabled" } else { "disabled" }
    );

    let decoding_key = match std::env::var("decoding_key") {
        Ok(key) => {
            tracing::info!("Using decoding key from environment");
            key
        }
        Err(_) => std::fs::read_to_string("./decoding_key")
            .inspect(|_| {
                tracing::info!("Using decoding key from filesystem");
            })
            .unwrap_or_else(|_| {
                tracing::error!("Decoding key was not found in either environment nor filesystem");
                tracing::info!("Fatal error encountered halting!");
                std::thread::park();
                panic!();
            }),
    };

    let bind_address = std::env::var("address").unwrap_or("0.0.0.0:80".into());

    HttpServer::new(move || {
        // TODO: Replace with actual key
        let jwt_decoding_key = jsonwebtoken::DecodingKey::from_secret(decoding_key.as_bytes());
        let mut jwt_validation = jsonwebtoken::Validation::default();
        jwt_validation.set_required_spec_claims(&["exp", "nbf"]);

        let req_client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
            .build()
            .unwrap();

        App::new()
            .wrap(JwtGrantsMiddleware::new(jwt_decoding_key, jwt_validation))
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(Logger::default())
            .wrap(Compress::default())
            .app_data(JsonConfig::default().error_handler(json_error::json_config_error_handler))
            .app_data(PathConfig::default().error_handler(json_error::json_config_error_handler))
            .app_data(Data::new(req_client))
            .app_data(GrantErrorConfig::<String>::default().error_handler(move |condition, grants| {
                use actix_web::ResponseError;

                if !is_debug_on {
                    return macros::resp_401_Unauthorized!();
                }

                let msg = format!(
                    "Insufficient permissions. Condition '{}' needs to be fulfilled. Grants provided: {:?}",
                    condition, grants
                );
                json_error::JsonError::new(msg, actix_web::http::StatusCode::FORBIDDEN).error_response()
            }))
            .configure(paths::configure)
            .default_service(if is_debug_on {
                web::to(default_handler_debug)
            } else {
                web::to(default_handler)
            })
    })
    .bind(bind_address)
    .expect("Failed to bind server to address")
    .run()
    .await
}
