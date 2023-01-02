use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_body: String,
    text_body: String,
}

pub struct EmailClient {
    client: Client,
    base_url: reqwest::Url,
    sender: SubscriberEmail,
    auth_token: Secret<String>,
}

impl EmailClient {
    pub fn new(base_url: &str, sender: SubscriberEmail, auth_token: Secret<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: reqwest::Url::parse(base_url).expect("unable to parse given url"),
            sender,
            auth_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = self
            .base_url
            .join("{}/email")
            .expect("Unable to `join` url");
        let request_body = SendEmailRequest {
            from: self.sender.as_ref().to_owned(),
            to: recipient.as_ref().to_owned(),
            subject: subject.to_owned(),
            html_body: html_content.to_owned(),
            text_body: text_content.to_owned(),
        };
        self.client
            .post(url)
            .header("X-Token", self.auth_token.expose_secret())
            .json(&request_body)
            .send()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    struct SendBodyMatcher;

    impl wiremock::Match for SendBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let fake_email: String = SafeEmail().fake();
        let sender = SubscriberEmail::parse(&fake_email).unwrap();
        let email_client = EmailClient::new(&mock_server.uri(), sender, Secret::new(Faker.fake()));

        Mock::given(header_exists("X-Token"))
            .and(header("Content-Type", "application/json"))
            .and(method("POST"))
            .and(SendBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let fake_email: String = SafeEmail().fake();
        let subscriber_email = SubscriberEmail::parse(&fake_email).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();
        let _ = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;
    }
}
