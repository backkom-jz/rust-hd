use actix_web::{HttpResponse, Responder};

pub async fn index_page() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../templates/index.html"))
}

pub async fn checkin_page() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../templates/checkin.html"))
}

pub async fn checkin2_page() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../templates/checkin2.html"))
}

pub async fn stats_page() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../templates/stats.html"))
}

pub async fn iching_page() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../templates/iching.html"))
}
