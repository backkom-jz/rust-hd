use actix_web::web;

mod checkin;
mod convert;
mod dino;
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
                .configure(dino::configure)
                .configure(geo::configure)
                .configure(convert::configure),
        );
}
