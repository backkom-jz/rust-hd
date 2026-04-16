use actix_web::{web, HttpResponse, Responder};

use crate::models::{CreateDinoScore, DinoScoreQuery};
use crate::services::DinoService;

fn is_bad_request(msg: &str) -> bool {
    matches!(
        msg,
        "昵称不能为空" | "昵称最多 16 个字符" | "分数不能小于 0" | "分数超过允许范围"
    )
}

pub async fn create_score(
    service: web::Data<DinoService>,
    body: web::Json<CreateDinoScore>,
) -> impl Responder {
    match service.create_score(body.into_inner()).await {
        Ok(()) => HttpResponse::Created().json(serde_json::json!({ "ok": true })),
        Err(msg) if is_bad_request(&msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn top_scores(
    service: web::Data<DinoService>,
    query: web::Query<DinoScoreQuery>,
) -> impl Responder {
    match service.top_scores(&query).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}
