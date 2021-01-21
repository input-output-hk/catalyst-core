use chain_vote::{
    committee::MemberSecretKey, MemberCommunicationKey, MemberPublicKey, MemberState, CRS,
};
use rand_core::CryptoRng;
use rand_core::RngCore;

pub struct CommitteeMembersManager {
    members: Vec<CommitteeMember>,
}

pub struct CommitteeMember {
    state: MemberState,
}

impl CommitteeMembersManager {
    pub fn new(rng: &mut (impl RngCore + CryptoRng), threshold: usize, members_no: usize) -> Self {
        let mut public_keys = Vec::new();
        for _ in 0..members_no {
            let private_key = MemberCommunicationKey::new(rng);
            let public_key = private_key.to_public();
            public_keys.push(public_key);
        }

        let crs = CRS::random(rng);

        let mut members = Vec::new();
        for i in 0..members_no {
            let state = MemberState::new(rng, threshold, &crs, &public_keys, i);
            members.push(CommitteeMember { state })
        }

        Self { members }
    }

    pub fn members(&self) -> &[CommitteeMember] {
        &self.members
    }
}

impl CommitteeMember {
    pub fn public_key(&self) -> MemberPublicKey {
        self.state.public_key()
    }

    pub fn secret_key(&self) -> &MemberSecretKey {
        self.state.secret_key()
    }
}
