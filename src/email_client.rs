use crate::domain::SubscriberEmail;
use reqwest::Client;

pub struct EmailClient {
    client: Client,
    base_url: reqwest::Url,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: &str, sender: SubscriberEmail) -> Self {
        Self {
            client: Client::new(),
            base_url: reqwest::Url::parse(base_url).expect("unable to parse given url"),
            sender,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        let url = self
            .base_url
            .join("{}/email")
            .expect("Unable to `join` url");
        let builder = self.client.post(url);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::Fake;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let fake_email: String = SafeEmail().fake();
        let sender = SubscriberEmail::parse(&fake_email).unwrap();
        let email_client = EmailClient::new(&mock_server.uri(), sender);

        Mock::given(any())
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
