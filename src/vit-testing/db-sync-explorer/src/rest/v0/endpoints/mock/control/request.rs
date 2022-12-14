use crate::mock::Actor;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResetRequest {
    pub actors: Vec<Actor>,
}
