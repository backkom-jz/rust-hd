use actix_web::{web, HttpResponse, Responder};

use crate::models::{CheckInDateQuery, CheckInRecord, SignInBody};
use crate::services::{CheckInService, ERR_CHECKIN_ALREADY_TODAY};

pub async fn fence_config(service: web::Data<CheckInService>) -> impl Responder {
    let f = service.get_fence().await;
    HttpResponse::Ok().json(serde_json::json!({
        "center_lat": f.center_lat,
        "center_lng": f.center_lng,
        "radius_km": f.radius_km,
        "venue": f.venue_name,
    }))
}

fn is_bad_request(msg: &str) -> bool {
    matches!(
        msg,
        "手机号不能为空" | "用户名不能为空" | "无效的日期" | "请提供有效的定位坐标"
    ) || msg.starts_with("签到地点需在")
}

fn respond_sign_in(result: Result<CheckInRecord, String>) -> HttpResponse {
    match result {
        Ok(record) => HttpResponse::Created().json(record),
        Err(msg) if msg == ERR_CHECKIN_ALREADY_TODAY => {
            HttpResponse::Conflict().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) if is_bad_request(&msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn sign_in(
    service: web::Data<CheckInService>,
    body: web::Json<SignInBody>,
) -> impl Responder {
    respond_sign_in(service.sign_in(body.into_inner()).await)
}

pub async fn sign_in_open(
    service: web::Data<CheckInService>,
    body: web::Json<SignInBody>,
) -> impl Responder {
    respond_sign_in(service.sign_in_open(body.into_inner()).await)
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
