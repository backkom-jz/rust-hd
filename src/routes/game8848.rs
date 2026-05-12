use actix_web::web;

use crate::handlers::game8848_handler;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/game8848")
            .route("/scores", web::get().to(game8848_handler::top_scores))
            .route("/scores", web::post().to(game8848_handler::create_score)),
    );
}
