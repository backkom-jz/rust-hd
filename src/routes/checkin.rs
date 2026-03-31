use actix_web::web;

use crate::handlers::checkin_handler;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/checkins")
            .route("/fence", web::get().to(checkin_handler::fence_config))
            .route("", web::post().to(checkin_handler::sign_in))
            .route("", web::get().to(checkin_handler::stats_by_date)),
    );
}
