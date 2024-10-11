use actix_web::web::ServiceConfig;

mod pokemon;

pub fn configure(cfg: &mut ServiceConfig) {
    pokemon::configure(cfg);
}
