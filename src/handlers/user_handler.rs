use actix_web::{web, HttpResponse, Responder};

use crate::models::CreateUser;
use crate::services::UserService;

pub async fn list_users(service: web::Data<UserService>) -> impl Responder {
    HttpResponse::Ok().json(service.list())
}

pub async fn get_user(service: web::Data<UserService>, path: web::Path<u64>) -> impl Responder {
    match service.get(*path) {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().finish(),
    }
}

pub async fn create_user(
    service: web::Data<UserService>,
    body: web::Json<CreateUser>,
) -> impl Responder {
    let user = service.create(body.into_inner());
    HttpResponse::Created().json(user)
}
