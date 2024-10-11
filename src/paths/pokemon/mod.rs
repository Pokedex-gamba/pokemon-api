mod get_by_name;

use actix_web::web::ServiceConfig;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(get_by_name::get_by_name);
}
