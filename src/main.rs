use actix_web::{
    http::StatusCode,
    middleware::{Compress, Logger, NormalizePath, TrailingSlash},
    web::{self, Data, JsonConfig, PathConfig},
    App, HttpServer,
};
use actix_web_grants::{GrantErrorConfig, GrantsConfig};
use docs::{AutoTagAddon, JwtGrantsAddon};
use empty_error::EmptyError;
use json_error::JsonError;
use jwt_stuff::JwtGrantsMiddleware;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use utoipauto::utoipauto;

mod docs;
mod empty_error;
mod json_error;
mod jwt_stuff;
mod macros;
mod models;
mod paths;
mod queries;
mod req_caching;

async fn default_handler_debug(req: actix_web::HttpRequest) -> impl actix_web::Responder {
    actix_web::HttpResponse::NotFound().body(format!("{:#?}", req))
}
async fn default_handler() -> impl actix_web::Responder {
    actix_web::HttpResponse::NotFound().finish()
}

static mut IS_DEBUG_ON: bool = false;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    #[utoipauto]
    #[derive(OpenApi)]
    #[openapi(
        info(
            title = "Pokemon API"
        ),
        modifiers(&JwtGrantsAddon, &AutoTagAddon)
    )]
    struct ApiDoc;

    let is_debug_on = std::env::var("DEBUG")
        .map(|val| val == "1")
        .unwrap_or_default();
    unsafe {
        IS_DEBUG_ON = is_debug_on;
    }
    tracing::info!(
        "Debug is {}",
        if is_debug_on { "enabled" } else { "disabled" }
    );

    let decoding_key = match std::env::var("DECODING_KEY") {
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

    let bind_address = std::env::var("ADDRESS").unwrap_or("0.0.0.0:80".into());

    HttpServer::new(move || {
        let jwt_decoding_key = match jsonwebtoken::DecodingKey::from_rsa_pem(decoding_key.as_bytes()) {
            Ok(key) => key,
            Err(e) => {
                tracing::error!("Parsing of decoding key failed with error: {}", e);
                tracing::info!("Fatal error encountered halting!");
                std::thread::park();
                panic!();
            },
        };
        let mut jwt_validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        jwt_validation.set_required_spec_claims(&["exp", "nbf"]);

        let req_client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
            .build()
            .unwrap();

        let json_config = JsonConfig::default().error_handler(
            if is_debug_on { json_error::config_json_error_handler } else { empty_error::config_empty_error_handler }
        );

        let path_config = PathConfig::default().error_handler(
            if is_debug_on { json_error::config_json_error_handler } else { empty_error::config_empty_error_handler }
        );

        let grants_string_error_config = GrantErrorConfig::<String>::default()
            .error_handler(move |condition, grants| {
                use actix_web::ResponseError;

                if !is_debug_on {
                    return macros::resp_401_Unauthorized!();
                }

                let msg = format!(
                    "Insufficient permissions. Condition '{}' needs to be fulfilled. Grants provided: {:?}",
                    condition, grants
                );
                json_error::JsonError::new(msg, actix_web::http::StatusCode::FORBIDDEN).error_response()
            });

        let grants_config = GrantsConfig::default().missing_auth_details_error_handler(move || {
            let code = StatusCode::UNAUTHORIZED;
            if is_debug_on { JsonError::new("Authorization header is missing".to_string(), code).into() } else { EmptyError::new(code).into() }
        });

        let jwt_grants_middleware = JwtGrantsMiddleware::new(
            jwt_decoding_key,
            jwt_validation,
        ).error_handler(move |error| {
            let code = StatusCode::BAD_REQUEST;
            if is_debug_on { JsonError::new(error.to_error_string(), code).into() } else { EmptyError::new(code).into() }
        });

        let mut app = App::new()
            .wrap(jwt_grants_middleware)
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(Logger::default())
            .wrap(Compress::default())
            .app_data(json_config)
            .app_data(path_config)
            .app_data(grants_config)
            .app_data(grants_string_error_config)
            .app_data(Data::new(req_client));
            if is_debug_on {
                app = app.service(Scalar::with_url("/docs", ApiDoc::openapi()));
            }
            app.configure(paths::configure)
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
