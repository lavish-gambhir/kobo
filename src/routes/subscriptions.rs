use actix_web::web::Form;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::types::chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );
    let _request_span_guard = request_span.enter();
    let query_span = tracing::info_span!("Saving a new subscriber details in the database");
    tracing::info!(
        "request_id {} -- adding '{}' '{}' as a new subscriber",
        request_id,
        form.email,
        form.name
    );
    tracing::info!(
        "request_id {} -- saving new subscriber details in the database",
        request_id
    );
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4) 
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.as_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} -- new subscriber details have been saved",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!(
                "request_id {} -- Failed to execute query: {:?}",
                e,
                request_id
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
