use actix_web::web;

use crate::handlers::pages;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/checkin", web::get().to(pages::checkin_page))
        .route("/checkin2", web::get().to(pages::checkin2_page))
        .route("/stats", web::get().to(pages::stats_page));
}
