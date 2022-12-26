use std::net::TcpListener;

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use kobo::configuration::get_configuration;

use kobo::startup;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let addr = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(addr).expect("unable to bind the address");
    println!(
        "server listening on port: {}",
        listener.local_addr().unwrap().port()
    );
    let _ = startup::run(listener)?.await;
    Ok(())
}
