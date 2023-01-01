use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;

use kobo::configuration::get_configuration;
use kobo::email_client::EmailClient;
use kobo::{startup, telemetry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    telemetry::init_subscriber(telemetry::get_subscriber("kobo".into(), "info".into()));

    let configuration = get_configuration().expect("Failed to read configuration");
    let addr = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(addr).expect("unable to bind the address");
    let connection = PgPoolOptions::new().connect_lazy_with(configuration.database.with_db());
    let email_client = EmailClient::new(
        &configuration.email_client.base_url,
        configuration
            .email_client
            .sender()
            .expect("Invalid sender email address"),
    );
    println!(
        "server listening on port: {:?}",
        listener.local_addr().unwrap()
    );
    let _ = startup::run(listener, connection, email_client)?.await;
    Ok(())
}
