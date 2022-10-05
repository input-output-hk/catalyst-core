use super::Error;
use crate::network::MainnetNetwork;
use crate::wallet::key::MainnetKey;
use snapshot_lib::registration::VotingRegistration;
pub struct RegistrationSender {
    voting_registration: VotingRegistration,
    key: Option<MainnetKey>,
}

impl RegistrationSender {
    pub fn new(voting_registration: VotingRegistration) -> Self {
        Self {
            voting_registration,
            key: None,
        }
    }

    pub fn sign_with(mut self, key: MainnetKey) -> Self {
        self.key = Some(key);
        self
    }

    pub fn to(self, network: &mut MainnetNetwork) -> Result<(), Error> {
        //this is just to simulate sending registration
        let _ = self.key.ok_or(Error::KeyNotProvided)?;

        network.accept(self.voting_registration);
        Ok(())
    }
}
