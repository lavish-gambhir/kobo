use crate::helpers::spawn_app;
use reqwest::Url;

use wiremock::matchers::{any, method};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_400() {
    let app = spawn_app().await;
    let response = reqwest::get(format!("{}/subscriptions/confirm", app.addr))
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(any())
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.to_string()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_link = app.get_confirmation_links(email_request);
    assert_eq!(confirmation_link.html.host_str().unwrap(), "127.0.0.1");

    let response = reqwest::get(confirmation_link.html).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_sub() {
    let app = spawn_app().await;
    let body = "name=luke%20skywalker&email=luke_skywalker%40gmail.com";

    Mock::given(any())
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.to_string()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subs");
    assert_eq!(saved.email, "luke_skywalker@gmail.com");
    assert_eq!(saved.name, "luke skywalker");
    assert_eq!(saved.status, "confirmed");
}
