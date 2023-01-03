use std::net::TcpListener;

use once_cell::sync::Lazy;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use kobo::configuration::{get_configuration, DatabaseSettings};
use kobo::email_client::EmailClient;
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

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let addr = format!("http://127.0.0.1:{}", port);
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&configuration.database).await;
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        &configuration.email_client.base_url,
        configuration
            .email_client
            .sender()
            .expect("Invalid sender email address"),
        configuration.email_client.auth_token,
        timeout,
    );
    let server = kobo::startup::run(listener, db_pool.clone(), email_client)
        .expect("failed to bind address");
    let _ = tokio::spawn(server);
    TestApp { addr, db_pool }
}
