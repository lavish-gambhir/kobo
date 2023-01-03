use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
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
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("unable to build `reqwest::Client`"),
            base_url: reqwest::Url::parse(base_url).expect("unable to parse given url"),
            sender,
            auth_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject_content: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = self
            .base_url
            .join("{}/email")
            .expect("Unable to `join` url");
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject: subject_content,
            html_body: html_content,
            text_body: text_content,
        };
        self.client
            .post(url)
            .header("X-Token", self.auth_token.expose_secret())
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::{assert_err, assert_ok};
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

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let mock_server = MockServer::start().await;
        let fake_email: String = SafeEmail().fake();
        let sender = SubscriberEmail::parse(&fake_email).unwrap();
        let email_client = EmailClient::new(&mock_server.uri(), sender, Secret::new(Faker.fake()));
        let fake_email: String = SafeEmail().fake();
        let subscriber_email = SubscriberEmail::parse(&fake_email).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(&SafeEmail().fake::<String>()).unwrap();
        let email_client = EmailClient::new(&mock_server.uri(), sender, Secret::new(Faker.fake()));
        let subscriber_email = SubscriberEmail::parse(&SafeEmail().fake::<String>()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let content = Paragraph(1..10).fake::<String>();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(&SafeEmail().fake::<String>()).unwrap();
        let email_client = EmailClient::new(&mock_server.uri(), sender, Secret::new(Faker.fake()));
        let subscriber_email = SubscriberEmail::parse(&SafeEmail().fake::<String>()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let content = Paragraph(1..10).fake::<String>();

        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;
        assert_err!(outcome);
    }
}
