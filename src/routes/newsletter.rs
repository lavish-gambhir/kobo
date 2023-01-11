use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use actix_web::body::BoxBody;
use actix_web::http::header::{HeaderMap, HeaderValue};
use actix_web::http::{header, StatusCode};
use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use anyhow::Context;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sha3::Digest;
use sqlx::PgPool;
use std::any::TypeId;
use std::error::Error;
use std::fmt::Formatter;
use std::str::ParseBoolError;

struct Credentials {
    username: String,
    password: Secret<String>,
}

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

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(request, body, pool, email_client),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    request: HttpRequest,
    body: web::Json<NewsletterBody>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, PublishError> {
    let creds = basic_authentication(request.headers()).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", &tracing::field::display(&creds.username));
    let user_id = validate_credentials(pool.as_ref(), creds).await?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
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

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF-8 string.")?;
    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_bytes = base64::decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not a valid UTF-8.")?;
    let mut creds = decoded_credentials.splitn(2, ":");
    let username = creds
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth"))?
        .to_string();
    let password = creds
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth"))?
        .to_string();
    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}

async fn validate_credentials(
    pool: &PgPool,
    credentials: Credentials,
) -> Result<uuid::Uuid, PublishError> {
    let current_span = tracing::Span::current();
    let row: Option<_> = sqlx::query!(
        r#"SELECT user_id, password_hash FROM editors WHERE username = $1"#,
        credentials.username
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query to retrieve stored credentials.")
    .map_err(PublishError::UnexpectedError)?;

    let (expected_password_hash, user_id) = match row {
        Some(row) => (row.password_hash, row.user_id),
        None => return Err(PublishError::AuthError(anyhow::anyhow!("Unknown username"))),
    };
    let expected_password_hash = PasswordHash::new(&expected_password_hash)
        .context("Failed to parse hash in PHC string format")
        .map_err(PublishError::UnexpectedError)?;
    let expected_password_hash = Secret::new(expected_password_hash.to_string());

    tokio::task::spawn_blocking(move || {
        let _ = current_span
            .in_scope(|| verify_password_hash(expected_password_hash, credentials.password));
    })
    .await
    .context("Failed to spawn blocking task")
    .map_err(PublishError::UnexpectedError)
    .context("Incorrect password")
    .map_err(PublishError::AuthError)?;

    Ok(user_id)
}

#[tracing::instrument(
    name = "Verify password hash"
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), PublishError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format")
        .map_err(PublishError::UnexpectedError)?;
    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password")
        .map_err(PublishError::AuthError)
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed.")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "caused by {:?}", self.source())
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
            Self::AuthError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                response
                    .headers_mut()
                    .insert(header::WWW_AUTHENTICATE, header_value);
                response
            }
        }
    }
}
