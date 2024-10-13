use actix_web::web::ServiceConfig;

pub mod pokemon;

pub fn configure(cfg: &mut ServiceConfig) {
    pokemon::configure(cfg);
}
