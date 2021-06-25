use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JobOutputInfo {
    pub checks: Checks,
    pub registration: RegistrationInfo,
    pub snapshot: SnapshotInfo,
}
