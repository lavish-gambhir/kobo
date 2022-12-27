use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;

use actix_web::{web, App, HttpServer};

use actix_web::middleware::Logger;
use sqlx::PgPool;
use std::net::TcpListener;

pub fn run(listener: TcpListener, pool: PgPool) -> Result<Server, std::io::Error> {
    // wrapping the `connection` in a smart pointer
    let pool = web::Data::new(pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
