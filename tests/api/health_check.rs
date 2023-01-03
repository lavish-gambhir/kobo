use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", app.addr))
        .send()
        .await
        .expect("failed to execute the request");
    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}
