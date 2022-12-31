use std::net::TcpListener;

use crate::email_client::EmailClient;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::routes::{health_check, subscribe};

pub fn run(
    listener: TcpListener,
    pool: PgPool,
    client: EmailClient,
) -> Result<Server, std::io::Error> {
    // wrapping the `connection` in a smart pointer
    let pool = web::Data::new(pool);
    let client = web::Data::new(client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(pool.clone())
            .app_data(client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
