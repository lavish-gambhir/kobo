use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: &str) -> Result<Self, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = s.chars().any(|c| forbidden_chars.contains(&c));
        if is_empty_or_whitespace || is_too_long || contains_forbidden_chars {
            Err(format!("{} is not a valid subscriber name", s))
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl AsRef<String> for SubscriberName {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a".repeat(256);
        assert_ok!(SubscriberName::parse(&name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(&name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ";
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "";
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_char_is_rejected() {
        for name in &['/', '(', ')', '"', '\\', '<', '>', '{', '}'] {
            assert_err!(SubscriberName::parse(&name.to_string()));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "r2f2";
        assert_ok!(SubscriberName::parse(name));
    }
}
