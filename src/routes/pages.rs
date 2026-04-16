use actix_web::web;

use crate::handlers::pages;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/", web::get().to(pages::index_page))
        .route("/calculator", web::get().to(pages::calculator_page))
        .route("/mortgage", web::get().to(pages::mortgage_page))
        .route("/dino", web::get().to(pages::dino_page))
        .route("/checkin", web::get().to(pages::checkin_page))
        .route("/checkin2", web::get().to(pages::checkin2_page))
        .route("/stats", web::get().to(pages::stats_page))
        .route("/iching", web::get().to(pages::iching_page))
        .route("/lottery", web::get().to(pages::lottery_page));
}
