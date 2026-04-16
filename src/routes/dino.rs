use actix_web::web;

use crate::handlers::dino_handler;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dino")
            .route("/scores", web::get().to(dino_handler::top_scores))
            .route("/scores", web::post().to(dino_handler::create_score)),
    );
}
