use serde::{de::Visitor, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeTitle(String);

impl<'de> Deserialize<'de> for ChallengeTitle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(V)
    }
}

struct V;

impl Visitor<'_> for V {
    type Value = ChallengeTitle;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "a string representing the title of a challenge (ignoring any leading `FX: `)",
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ChallengeTitle::new(v))
    }
}

impl ChallengeTitle {
    pub fn new(s: &str) -> Self {
        let s = s.trim_start_matches("FX: ");
        Self(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<ChallengeTitle> for String {
    fn from(ChallengeTitle(inner): ChallengeTitle) -> Self {
        inner
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn parse(s: &str) -> ChallengeTitle {
        let json = json!(s);
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn strips_leading_fx() {
        assert_eq!(parse("hello"), ChallengeTitle("hello".into()));
        assert_eq!(parse("FX: hello"), ChallengeTitle("hello".into()));
        assert_eq!(parse("FX:hello"), ChallengeTitle("FX:hello".into()));
    }
}
