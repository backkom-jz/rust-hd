use actix_web::web;

use crate::handlers::convert_handler;

const MAX_PAYLOAD: usize = 20 * 1024 * 1024 + 65536;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/convert")
            .app_data(web::PayloadConfig::default().limit(MAX_PAYLOAD))
            .route(web::post().to(convert_handler::convert_image)),
    );
}
