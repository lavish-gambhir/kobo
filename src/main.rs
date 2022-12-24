use std::net::TcpListener;

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};

use kobo::startup;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("unable to bind the address");
    println!(
        "server listening on port: {}",
        listener.local_addr().unwrap().port()
    );
    let _ = startup::run(listener)?.await;
    Ok(())
}
