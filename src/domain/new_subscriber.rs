use crate::domain::SubscriberName;

#[derive(Debug)]
pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

impl NewSubscriber {
    pub fn new(email: &str, name: SubscriberName) -> Self {
        Self {
            email: email.to_string(),
            name,
        }
    }
}
