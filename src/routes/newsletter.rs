use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use serde::Deserialize;
use sqlx::PgPool;
use std::any::TypeId;
use std::error::Error;
use std::fmt::Formatter;

struct ConfirmedSubscriber {
    email: SubscriberEmail,
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
    body: web::Json<NewsletterBody>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, PublishError> {
    let confirmed_subscribers = get_confirmed_subscribers(pool.as_ref()).await?;
    for subscriber in confirmed_subscribers {
        email_client
            .send_email(
                &subscriber.email,
                &body.title,
                &body.content.html,
                &body.content.text,
            )
            .await
            .with_context(|| {
                format!("Failed to send newsletter issue to {:?}", subscriber.email)
            })?;
    }
    Ok(HttpResponse::Ok().finish())
}

async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    let confirmed_subscribers =
        sqlx::query!(r#"SELECT email FROM subscriptions WHERE status = 'confirmed'"#,)
            .fetch_all(pool)
            .await?
            .into_iter()
            .filter_map(|r| match SubscriberEmail::parse(&r.email) {
                Ok(email) => Some(ConfirmedSubscriber { email }),
                Err(e) => {
                    tracing::warn!(
                        "A confirmed subscriber is using an invalid email address:{}",
                        e
                    );
                    None
                }
            })
            .collect();
    Ok(confirmed_subscribers)
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
