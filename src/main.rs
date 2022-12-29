use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;

use kobo::configuration::get_configuration;
use kobo::{startup, telemetry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    telemetry::init_subscriber(telemetry::get_subscriber("kobo".into(), "info".into()));

    let configuration = get_configuration().expect("Failed to read configuration");
    let addr = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(addr).expect("unable to bind the address");
    let connection = PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await
        .expect("failed to connect to Postgres");
    println!(
        "server listening on port: {}",
        listener.local_addr().unwrap().port()
    );
    let _ = startup::run(listener, connection)?.await;
    Ok(())
}
