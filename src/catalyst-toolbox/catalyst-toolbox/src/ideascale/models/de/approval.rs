use serde::{de::Visitor, Deserialize, Deserializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Approval {
    Approved,
    NotApproved,
}

impl Approval {
    pub fn as_bool(&self) -> bool {
        (*self).into()
    }
}

impl From<bool> for Approval {
    fn from(b: bool) -> Self {
        match b {
            true => Approval::Approved,
            false => Approval::NotApproved,
        }
    }
}

impl From<Approval> for bool {
    fn from(b: Approval) -> Self {
        match b {
            Approval::Approved => true,
            Approval::NotApproved => false,
        }
    }
}

impl<'de> Deserialize<'de> for Approval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(V)
    }
}

struct V;

impl Visitor<'_> for V {
    type Value = Approval;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representing an approval status (with ths string \"approved\" meaning `Approved`, and all other strings being `NotApproved`)")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "approved" => Ok(Approval::Approved),
            _ => Ok(Approval::NotApproved),
        }
    }
}
