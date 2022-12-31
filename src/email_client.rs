use crate::domain::SubscriberEmail;
use reqwest::Client;

pub struct EmailClient {
    client: Client,
    base_url: String,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: &str, sender: SubscriberEmail) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
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
        todo!()
    }
}
