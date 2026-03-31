use actix_web::web;

mod checkin;
mod geo;
mod pages;
mod user;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .configure(pages::configure)
        .service(
            web::scope("/api")
                .configure(user::configure)
                .configure(checkin::configure)
                .configure(geo::configure),
        );
}
