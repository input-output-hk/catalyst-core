use jormungandr_lib::interfaces::BlockDate;
use snapshot_lib::registration::VotingRegistration;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Default)]
pub struct DbSyncInstance {
    state: HashMap<BlockDate, Vec<VotingRegistration>>,
    settings: Settings,
}

impl DbSyncInstance {
    pub fn notify(&mut self, block_date: BlockDate, registration: VotingRegistration) {
        match self.state.entry(block_date) {
            Entry::Vacant(e) => {
                e.insert(vec![registration]);
            }
            Entry::Occupied(mut e) => {
                e.get_mut().push(registration);
            }
        }
    }

    pub fn query_all_registration_transactions(&self) -> Vec<VotingRegistration> {
        self.state
            .values()
            .cloned()
            .fold(vec![], |mut vec, mut value| {
                vec.append(&mut value);
                vec
            })
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub db_name: String,
    pub db_user: String,
    pub db_host: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            db_name: "mock".to_string(),
            db_user: "mock".to_string(),
            db_host: "mock".to_string(),
        }
    }
}
