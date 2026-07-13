use actix_web::{web, HttpResponse, Responder};

use crate::models::CreateColorDrawBatch;
use crate::services::ColorDrawService;

fn is_bad_request(msg: &str) -> bool {
    msg.contains("名字")
        || msg.contains("批次")
        || msg.contains("剩余颜色")
        || msg.contains("已抽完")
        || msg.contains("进行中")
}

pub async fn current_session(service: web::Data<ColorDrawService>) -> impl Responder {
    match service.current_session().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn create_batch(
    service: web::Data<ColorDrawService>,
    body: web::Json<CreateColorDrawBatch>,
) -> impl Responder {
    match service.create_batch(body.into_inner()).await {
        Ok(batch) => HttpResponse::Created().json(batch),
        Err(msg) if is_bad_request(&msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn draw_next(service: web::Data<ColorDrawService>) -> impl Responder {
    match service.draw_next().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(msg) if is_bad_request(&msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn reset_session(service: web::Data<ColorDrawService>) -> impl Responder {
    match service.abandon_open().await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "ok": true })),
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}
