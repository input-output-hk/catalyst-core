use crate::vote::VotePlanStatus;
use chain_vote::{
    committee::MemberSecretKey, Crs, ElectionPublicKey, MemberCommunicationKey, MemberPublicKey,
    MemberState, TallyDecryptShare,
};
use rand::thread_rng;
use rand_core::CryptoRng;
use rand_core::RngCore;

pub struct CommitteeMembersManager {
    members: Vec<CommitteeMember>,
    crs: Crs,
}

pub struct CommitteeMember {
    state: MemberState,
}

impl CommitteeMembersManager {
    pub fn new(
        rng: &mut (impl RngCore + CryptoRng),
        crs_seed: &[u8],
        threshold: usize,
        members_no: usize,
    ) -> Self {
        let mut public_keys = Vec::new();
        for _ in 0..members_no {
            let private_key = MemberCommunicationKey::new(rng);
            let public_key = private_key.to_public();
            public_keys.push(public_key);
        }

        let crs = Crs::from_hash(&crs_seed);

        let mut members = Vec::new();
        for i in 0..members_no {
            let state = MemberState::new(rng, threshold, &crs, &public_keys, i);
            members.push(CommitteeMember { state })
        }

        Self { members, crs }
    }

    pub fn members(&self) -> &[CommitteeMember] {
        &self.members
    }

    pub fn election_pk(&self) -> ElectionPublicKey {
        let keys: Vec<_> = self.members().iter().map(|x| x.public_key()).collect();
        ElectionPublicKey::from_participants(&keys)
    }

    pub fn crs(&self) -> &Crs {
        &self.crs
    }
}

impl CommitteeMember {
    pub fn public_key(&self) -> MemberPublicKey {
        self.state.public_key()
    }

    pub fn secret_key(&self) -> &MemberSecretKey {
        self.state.secret_key()
    }

    pub fn produce_decrypt_shares(
        &self,
        vote_plan_status: &VotePlanStatus,
    ) -> Vec<TallyDecryptShare> {
        vote_plan_status
            .proposals
            .iter()
            .map(|proposal| {
                let tally_state = proposal.tally.as_ref().unwrap();
                let encrypted_tally = tally_state.private_encrypted().unwrap().0.clone();
                encrypted_tally.partial_decrypt(&mut thread_rng(), self.secret_key())
            })
            .collect()
    }
}
