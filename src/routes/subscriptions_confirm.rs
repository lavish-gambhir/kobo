use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "confirm a pending subscriber", skip(_params))]
pub async fn confirm_sub(_params: web::Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
