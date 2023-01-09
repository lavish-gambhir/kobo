use crate::helpers::{spawn_app, TestApp};
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    let _ = create_unconfirmed_subscribers(&app);

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
          "title": "Newsletter title",
          "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body</p>",
        }
    });

    let response = reqwest::Client::new()
        .post(format!("{}/newsletter", &app.addr))
        .json(&newsletter_request_body)
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status().as_u16(), 200);
}

async fn create_unconfirmed_subscribers(app: &TestApp) {
    let body = "name=john%20doe&email=john_doe%40gmail.com";
    let _mock_guard = Mock::given(any())
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create a new unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.to_string())
        .await
        .error_for_status()
        .unwrap();
}
