use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;

use kobo::configuration::get_configuration;
use kobo::email_client::EmailClient;
use kobo::startup::Application;
use kobo::{startup, telemetry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    telemetry::init_subscriber(telemetry::get_subscriber("kobo".into(), "info".into()));

    let configuration = get_configuration().expect("Failed to read configuration");
    let application = Application::build(&configuration)
        .await
        .expect("unable to build app");

    println!("server listening on port: {:?}", application.port());
    let _ = application.run_until_stopped().await;
    Ok(())
}
