use actix_web::{
    middleware::{Compress, Logger, NormalizePath, TrailingSlash},
    web::{self, JsonConfig, PathConfig},
    App, HttpServer,
};
use jwt_stuff::JwtGrantsMiddleware;

mod json_error;
mod jwt_stuff;

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

        App::new()
            .wrap(JwtGrantsMiddleware::new(jwt_decoding_key, jwt_validation))
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(Logger::default())
            .wrap(Compress::default())
            .app_data(JsonConfig::default().error_handler(json_error::json_config_error_handler))
            .app_data(PathConfig::default().error_handler(json_error::json_config_error_handler))
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
