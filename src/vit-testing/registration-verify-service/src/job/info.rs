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
    pub slot_no: Option<u64>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Checks {
    asserts: Vec<Assert>,
    passed: bool,
}

impl Checks {
    pub fn push(&mut self, assert: Assert) {
        self.asserts.push(assert);
    }

    pub fn calculate_passed(&mut self) {
        self.passed = !self
            .asserts
            .iter()
            .any(|x| matches!(x, Assert::Failed { .. }));
    }
}
impl Default for Checks {
    fn default() -> Self {
        Self {
            asserts: vec![],
            passed: true,
        }
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
