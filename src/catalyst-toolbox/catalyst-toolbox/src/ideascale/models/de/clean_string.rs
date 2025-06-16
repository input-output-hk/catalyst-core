use std::fmt::Formatter;

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer,
};

/// A newtype wrapper around `String`
///
/// When deserialized, the following characters are removed: `-`, `*`, `/`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanString(pub String);

impl<'de> Deserialize<'de> for CleanString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(V)
    }
}

struct V;

impl<'a> Visitor<'a> for V {
    type Value = CleanString;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let s = clean_str(v);
        Ok(CleanString(s))
    }
}

impl From<&str> for CleanString {
    fn from(s: &str) -> Self {
        CleanString(s.to_string())
    }
}

impl From<String> for CleanString {
    fn from(s: String) -> Self {
        CleanString(s)
    }
}

impl ToString for CleanString {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl AsRef<str> for CleanString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("[-*/]").unwrap());

pub fn clean_str(s: &str) -> String {
    REGEX.replace_all(s, "").to_string()
}

#[cfg(any(test, feature = "test-api"))]
mod tests {
    use proptest::arbitrary::any;
    use proptest::prelude::*;
    use proptest::{
        arbitrary::{Arbitrary, StrategyFor},
        prelude::*,
        strategy::Map,
    };
    use serde_json::json;
    use test_strategy::proptest;

    use super::*;

    impl Arbitrary for CleanString {
        type Parameters = ();
        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<String>().prop_map(|s| CleanString(clean_str(&s)))
        }

        type Strategy = Map<StrategyFor<String>, fn(String) -> Self>;
    }

    #[allow(dead_code)]
    fn parse(s: &str) -> CleanString {
        let s = format!(r#""{s}""#);
        serde_json::from_str(&s).unwrap()
    }

    #[test]
    fn correctly_formats_strings() {
        assert_eq!(parse("hello"), CleanString::from("hello"));
        assert_eq!(parse("h*e-l/lo"), CleanString::from("hello"));
    }

    proptest! {
        #[test]
        fn any_string_deserializes_to_clean_string(s in any::<String>()) {
            let json = json!(s);
            let _: CleanString = serde_json::from_value(json).unwrap();
        }
    }
}
