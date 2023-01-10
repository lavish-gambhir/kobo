use crate::helpers::{spawn_app, ConfirmationLinks, TestApp};
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

    let response = app.post_newsletters(newsletter_request_body).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletter_is_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    let _ = create_confirmed_subs(&app);
    Mock::given(any())
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
          "title": "Newsletter title",
          "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body</p>",
        }
    });

    let response = app.post_newsletters(newsletter_request_body).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletter_returns_400_for_invalid_data() {
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "content": {
                    "text": "Newsletter body",
                    "html": "<p>Newsletter body</p>",
                }
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "newsletter!"
            }),
            "missing content",
        ),
    ];

    for (invalid_body, err_msg) in test_cases {
        let response = app.post_newsletters(invalid_body).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with a 400 BAD REQUEST when the payload was {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    let app = spawn_app().await;
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
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

async fn create_unconfirmed_subscribers(app: &TestApp) -> ConfirmationLinks {
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
    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(email_request)
}

async fn create_confirmed_subs(app: &TestApp) {
    let confirmation_links = create_unconfirmed_subscribers(app).await;
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .expect("Unable to send confirmation request");
}
