mod get_all;
mod get_by_name;
mod get_random;

use actix_web::web::ServiceConfig;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(get_by_name::get_by_name)
        .service(get_all::get_all)
        .service(get_random::get_random);
}
