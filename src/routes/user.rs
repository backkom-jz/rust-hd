use actix_web::web;

use crate::handlers::user_handler;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("", web::get().to(user_handler::list_users))
            .route("", web::post().to(user_handler::create_user))
            .route("/{id}", web::get().to(user_handler::get_user)),
    );
}
