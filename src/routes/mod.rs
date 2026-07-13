use actix_web::web;

mod checkin;
mod color_draw;
mod convert;
mod dino;
mod game8848;
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
                .configure(game8848::configure)
                .configure(geo::configure)
                .configure(convert::configure)
                .configure(color_draw::configure),
        );
}
