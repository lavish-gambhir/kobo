
use std::net::TcpListener;


use env_logger::Env;
use kobo::configuration::get_configuration;
use sqlx::{Connection, PgPool};

use kobo::startup;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let configuration = get_configuration().expect("Failed to read configuration");
    let addr = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(addr).expect("unable to bind the address");
    let connection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("failed to connect to Postgres");
    println!(
        "server listening on port: {}",
        listener.local_addr().unwrap().port()
    );
    let _ = startup::run(listener, connection)?.await;
    Ok(())
}
