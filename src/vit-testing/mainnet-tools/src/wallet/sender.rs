use crate::network::MainnetNetwork;
use catalyst_toolbox::snapshot::VotingRegistration;

pub struct RegistrationSender {
    voting_registration: VotingRegistration,
}

impl RegistrationSender {
    pub fn new(voting_registration: VotingRegistration) -> Self {
        Self {
            voting_registration,
        }
    }

    pub fn to(self, network: &mut MainnetNetwork) {
        network.accept(self.voting_registration);
    }
}
