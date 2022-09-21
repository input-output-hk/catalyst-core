use crate::db_sync::DbSyncInstance;
use jormungandr_lib::interfaces::BlockDate;
use snapshot_lib::registration::VotingRegistration;

pub struct MainnetNetwork<'a> {
    block_date: BlockDate,
    observers: Vec<&'a mut DbSyncInstance>,
}

impl Default for MainnetNetwork<'_> {
    fn default() -> Self {
        Self {
            block_date: BlockDate::new(0, 0),
            observers: vec![],
        }
    }
}

impl<'a> MainnetNetwork<'a> {
    pub fn accept(&mut self, registration: VotingRegistration) {
        self.notify_all(self.block_date, registration);
    }

    pub fn sync_with(&mut self, observer: &'a mut DbSyncInstance) {
        self.observers.push(observer);
    }

    fn notify_all(&mut self, block_date: BlockDate, registration: VotingRegistration) {
        for observer in &mut self.observers {
            observer.notify(block_date, registration.clone());
        }
    }
}
