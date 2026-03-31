use actix_web::web;

use crate::handlers::geo_handler;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/geo").route("/reverse", web::get().to(geo_handler::reverse_geocode)));
}
