mod config;
mod coord_cn;
mod db;
mod geo;
mod handlers;
mod models;
mod routes;
mod services;

use actix_files::Files;
use actix_web::{web, App, HttpServer};
use sqlx::mysql::MySqlPoolOptions;

use crate::services::{CheckInService, ColorDrawService, DinoService, Game8848Service, UserService};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = config::database_url().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("database configuration: {e}"),
        )
    })?;

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("MySQL connect failed: {e}"),
            )
        })?;

    db::init_schema(&pool).await.map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("database init failed: {e}"),
        )
    })?;

    let user_service = UserService::new();
    let checkin_service = CheckInService::new(pool.clone());
    let dino_service = DinoService::new(pool.clone());
    let game8848_service = Game8848Service::new(pool.clone());
    let color_draw_service = ColorDrawService::new(pool);
    let bind_addr = config::bind_addr();
    let static_dir = config::static_dir();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(user_service.clone()))
            .app_data(web::Data::new(checkin_service.clone()))
            .app_data(web::Data::new(dino_service.clone()))
            .app_data(web::Data::new(game8848_service.clone()))
            .app_data(web::Data::new(color_draw_service.clone()))
            .service(Files::new("/static", static_dir.clone()).prefer_utf8(true))
            .configure(routes::config)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
