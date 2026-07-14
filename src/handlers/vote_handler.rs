use actix_web::{web, HttpResponse, Responder};

use crate::models::{CreateVotePoll, SubmitVote, VotePhoneQuery, VoteResultsQuery};
use crate::services::VoteService;

fn is_bad_request(msg: &str) -> bool {
    msg.contains("标题")
        || msg.contains("选项")
        || msg.contains("手机号")
        || msg.contains("投票")
        || msg.contains("场次")
        || msg.contains("无效")
}

pub async fn current_public(service: web::Data<VoteService>) -> impl Responder {
    match service.current_public().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn vote_status(
    service: web::Data<VoteService>,
    query: web::Query<VotePhoneQuery>,
) -> impl Responder {
    match service.vote_status(&query).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(msg) if is_bad_request(&msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn submit(
    service: web::Data<VoteService>,
    body: web::Json<SubmitVote>,
) -> impl Responder {
    match service.submit(body.into_inner()).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(msg) if is_bad_request(&msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn user_results(
    service: web::Data<VoteService>,
    query: web::Query<VoteResultsQuery>,
) -> impl Responder {
    match service.user_results(&query).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(msg) if is_bad_request(&msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn admin_results(
    service: web::Data<VoteService>,
    query: web::Query<VoteResultsQuery>,
) -> impl Responder {
    match service.admin_results(query.poll_id).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(msg) if is_bad_request(&msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn history(service: web::Data<VoteService>) -> impl Responder {
    match service.history().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn create_poll(
    service: web::Data<VoteService>,
    body: web::Json<CreateVotePoll>,
) -> impl Responder {
    match service.create_poll(body.into_inner()).await {
        Ok(data) => HttpResponse::Created().json(data),
        Err(msg) if is_bad_request(&msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}

pub async fn close_current(service: web::Data<VoteService>) -> impl Responder {
    match service.close_current().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(msg) if is_bad_request(&msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({ "error": msg }))
        }
        Err(msg) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": msg })),
    }
}
