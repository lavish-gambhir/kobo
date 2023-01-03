use std::net::TcpListener;

use once_cell::sync::Lazy;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;

use kobo::configuration::{get_configuration, DatabaseSettings};
use kobo::email_client::EmailClient;
use kobo::startup::{get_connection_pool, Application};
use kobo::telemetry;

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = telemetry::get_subscriber("test".into(), "debug".into());
    telemetry::init_subscriber(subscriber);
});

pub struct TestApp {
    pub addr: String,
    pub db_pool: PgPool,
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // create db
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    let _ = connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // migrate db
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");
    let _ = sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub async fn spawn_app() -> TestApp {
    // Uncomment this if you want tracing logs.
    // Need to add a sink for debug
    // Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        // Use the mock server as email API
        c.email_client.base_url = email_server.uri();
        c
    };

    // Create and migrate the database
    configure_database(&configuration.database).await;
    let application = Application::build(&configuration)
        .await
        .expect("unable to build app");
    let addr = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());
    TestApp {
        addr,
        db_pool: get_connection_pool(&configuration.database),
    }
}
