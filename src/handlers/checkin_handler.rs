use actix_web::{web, HttpResponse, Responder};

use crate::models::{CheckInDateQuery, SignInBody};
use crate::services::CheckInService;

fn is_bad_request(msg: &str) -> bool {
    matches!(
        msg,
        "手机号不能为空" | "用户名不能为空" | "无效的日期"
    )
}

pub async fn sign_in(
    service: web::Data<CheckInService>,
    body: web::Json<SignInBody>,
) -> impl Responder {
    match service.sign_in(body.into_inner()).await {
        Ok(record) => HttpResponse::Created().json(record),
        Err(msg) if is_bad_request(&msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn stats_by_date(
    service: web::Data<CheckInService>,
    query: web::Query<CheckInDateQuery>,
) -> impl Responder {
    match service.stats_for_query(&query).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(msg) if is_bad_request(&msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}
