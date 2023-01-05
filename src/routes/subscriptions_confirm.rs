use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "confirm a pending subscriber", skip(params, pool))]
pub async fn confirm_sub(params: web::Query<Parameters>, pool: web::Data<PgPool>) -> HttpResponse {
    let subscriber_id = match get_subscriber_id_from_token(&pool, &params.subscription_token).await
    {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    match subscriber_id {
        None => HttpResponse::Unauthorized().finish(),
        Some(id) => {
            if confirm_subscriber_id(pool.as_ref(), id).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
    }
}

#[tracing::instrument(name = "Get subscriber_id from subscription_token", skip(pool, token))]
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        token,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("failed to execute query: {:?}", e);
        e
    })?;
    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(
    name = "Mark a new subscriber as confirmed in db",
    skip(pool, subscriber_id)
)]
async fn confirm_subscriber_id(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
