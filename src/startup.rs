use std::net::{TcpListener};

use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::routes::{health_check, subscribe};

pub struct Application {
    server: Server,
    port: u16,
}

impl Application {
    pub async fn build(configuration: &Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let sender_email = configuration
            .email_client
            .sender()
            .expect("invalid sender email address");
        let timeout = configuration.email_client.timeout();
        let auth_token = configuration.email_client.auth_token.clone();
        let email_client = EmailClient::new(
            &configuration.email_client.base_url,
            sender_email,
            auth_token,
            timeout,
        );
        let addr = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let lstnr = TcpListener::bind(addr)?;
        let port = lstnr.local_addr().unwrap().port();
        let server = Self::run(lstnr, connection_pool, email_client).await?;
        Ok(Self { server, port })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }

    async fn run(
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
}

pub fn get_connection_pool(db_settings: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(db_settings.with_db())
}
