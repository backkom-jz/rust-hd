use actix_web::web;

use crate::handlers::color_draw_handler;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/color-draw")
            .route("/session", web::get().to(color_draw_handler::current_session))
            .route("/session", web::post().to(color_draw_handler::create_batch))
            .route("/draw", web::post().to(color_draw_handler::draw_next))
            .route("/reset", web::post().to(color_draw_handler::reset_session)),
    );
}
