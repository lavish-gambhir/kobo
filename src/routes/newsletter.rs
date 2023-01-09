use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use serde::Deserialize;
use sqlx::PgPool;
use std::any::TypeId;
use std::error::Error;
use std::fmt::Formatter;

struct ConfirmedSubscriber {
    email: String,
}

#[derive(Deserialize)]
pub struct NewsletterBody {
    title: String,
    content: Content,
}

#[derive(Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

pub async fn publish_newsletter(
    _body: web::Json<NewsletterBody>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, anyhow::Error> {
    let _confirmed_subscribers = get_confirmed_subscribers(pool.as_ref()).await?;
    Ok(HttpResponse::Ok().finish())
}

async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    let rows = sqlx::query_as!(
        ConfirmedSubscriber,
        r#"SELECT email FROM subscriptions WHERE status = 'confirmed'"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "caused by {:?}", self.source())
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
