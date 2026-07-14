use actix_web::web;

use crate::handlers::vote_handler;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/vote")
            .route("/current", web::get().to(vote_handler::current_public))
            .route("/status", web::get().to(vote_handler::vote_status))
            .route("/submit", web::post().to(vote_handler::submit))
            .route("/results", web::get().to(vote_handler::user_results))
            .route("/admin/results", web::get().to(vote_handler::admin_results))
            .route("/admin/history", web::get().to(vote_handler::history))
            .route("/admin/polls", web::post().to(vote_handler::create_poll))
            .route("/admin/close", web::post().to(vote_handler::close_current)),
    );
}
