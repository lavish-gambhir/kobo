use unicode_segmentation::UnicodeSegmentation;

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

pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: &str) -> Self {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = s.chars().any(|c| forbidden_chars.contains(&c));
        if is_empty_or_whitespace || is_too_long || contains_forbidden_chars {
            panic!("{} is not a valid subscriber name", s)
        } else {
            Self(s.to_string())
        }
    }
}

impl AsRef<String> for SubscriberName {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
