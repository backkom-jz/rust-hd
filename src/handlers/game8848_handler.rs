use actix_web::{web, HttpResponse, Responder};

use crate::models::{CreateGame8848Score, Game8848ScoreQuery};
use crate::services::Game8848Service;

fn is_bad_request(msg: &str) -> bool {
    matches!(
        msg,
        "昵称不能为空" | "昵称最多 16 个字符" | "高度不能为负数" | "高度超过允许范围" | "游戏时长超过允许范围"
    )
}

pub async fn create_score(
    service: web::Data<Game8848Service>,
    body: web::Json<CreateGame8848Score>,
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
    service: web::Data<Game8848Service>,
    query: web::Query<Game8848ScoreQuery>,
) -> impl Responder {
    match service.top_scores(&query).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}
