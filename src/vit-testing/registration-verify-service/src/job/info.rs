use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JobOutputInfo {
    pub checks: Checks,
    pub registration: RegistrationInfo,
    pub snapshot: SnapshotInfo,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct RegistrationInfo {
    pub expected_funds: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct SnapshotInfo {
    pub threshold: u64,
    pub slot_no: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Checks(Vec<Assert>);

impl Checks {
    pub fn push(&mut self, assert: Assert) {
        self.0.push(assert);
    }
}
impl Default for Checks {
    fn default() -> Self {
        Self(vec![])
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum Assert {
    Passed(String),
    Failed(String),
}

impl Assert {
    pub fn from_eq<A: Eq + ToString, S: Into<String>>(
        left: A,
        right: A,
        passed_comment: S,
        failed_comment: S,
    ) -> Assert {
        match left == right {
            true => Assert::Passed(passed_comment.into()),
            false => Assert::Failed(failed_comment.into()),
        }
    }
}
