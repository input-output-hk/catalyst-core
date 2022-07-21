use std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
};

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct AdaRewards(pub u64);

impl<'de> Deserialize<'de> for AdaRewards {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let num = deserializer.deserialize_str(V)?;
        Ok(Self(num))
    }
}

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\$([0-9]+) in ada"#).unwrap());

struct V;

impl<'a> Visitor<'a> for V {
    type Value = u64;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "a string of the form: `$N in ada`, where `N` is a u64 (e.g. \"$123 in ada\")",
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        // input is not standarized, hack an early return if it is just 0 ada
        if v.starts_with("0 ada") {
            return Ok(0);
        }

        let bad_pattern = || E::custom("didn't match `$N in ada` pattern");
        let bad_u64 = |e: ParseIntError| E::custom("unvalid u64: {e}");

        // ignore the first capture, since this is the whole string
        let capture = REGEX.captures_iter(v).next().ok_or_else(bad_pattern)?;
        let value = capture.get(1).ok_or_else(bad_pattern)?;
        value.as_str().parse().map_err(bad_u64)
    }
}

impl From<u64> for AdaRewards {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<AdaRewards> for u64 {
    fn from(rewards: AdaRewards) -> Self {
        rewards.0
    }
}

impl Display for AdaRewards {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "${} in ada", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Result<AdaRewards, serde_json::Error> {
        let s = format!(r#""{s}""#);
        serde_json::from_str(&s)
    }

    #[test]
    fn can_parse_good_values() {
        assert_eq!(parse("$123 in ada").unwrap(), AdaRewards(123));
        assert_eq!(parse("0 ada").unwrap(), AdaRewards(0));
        assert_eq!(
            parse("0 ada with some stuff at the end").unwrap(),
            AdaRewards(0)
        );
    }

    #[test]
    fn fails_to_parse_bad_values() {
        // missing dollar sign
        assert!(parse("123 in ada").is_err());
        // negative number
        assert!(parse("$-123 in ada").is_err());
        // fraction
        assert!(parse("$123.0 in ada").is_err());
    }
}

// fn deserialize_rewards<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
//     let rewards_str = String::deserialize(deserializer)?;
//
//     if rewards_str.starts_with("0 ada") {
//         return Ok(0);
//     }
//     sscanf::scanf!(rewards_str.trim_end(), "${} in {}", String, String)
//         // trim all . or , in between numbers
//         .map(|(mut amount, _currency)| {
//             amount.retain(|c: char| c.is_numeric() && !(matches!(c, '.') || matches!(c, ',')));
//             amount
//         })
//         .and_then(|s| s.parse().ok())
//         .ok_or_else(|| {
//             D::Error::custom(&format!(
//                 "Unable to read malformed value: '{}'",
//                 rewards_str
//             ))
//         })
// }
