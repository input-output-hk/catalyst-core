use mainnet_lib::wallet_state::Actor;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResetRequest {
    pub actors: Vec<Actor>,
}
