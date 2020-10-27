use chain_impl_mockchain::{
    certificate::{VoteCast, VotePlanId},
    vote::{Choice, Options, Payload},
};
pub use chain_vote::EncryptingVoteKey;
use chain_vote::Vote;

pub const VOTE_PLAN_ID_LENGTH: usize = 32;

pub struct Proposal {
    vote_plan_id: VotePlanId,
    index: u8,
    options: Options,
    payload_type: PayloadTypeConfig,
}

pub enum PayloadTypeConfig {
    Public,
    Private(chain_vote::EncryptingVoteKey),
}

impl Proposal {
    pub fn new(
        vote_plan_id: VotePlanId,
        index: u8,
        options: Options,
        payload_type: PayloadTypeConfig,
    ) -> Self {
        Self {
            vote_plan_id,
            payload_type,
            index,
            options,
        }
    }

    pub fn new_public(vote_plan_id: VotePlanId, index: u8, options: Options) -> Self {
        Self::new(vote_plan_id, index, options, PayloadTypeConfig::Public)
    }

    pub fn new_private(
        vote_plan_id: VotePlanId,
        index: u8,
        options: Options,
        key: EncryptingVoteKey,
    ) -> Self {
        Self::new(
            vote_plan_id,
            index,
            options,
            PayloadTypeConfig::Private(key),
        )
    }

    pub fn vote(&self, choice: Choice) -> Option<VoteCast> {
        if !self.options.validate(choice) {
            return None;
        }

        let payload = match self.payload_type {
            PayloadTypeConfig::Public => Payload::Public { choice },
            PayloadTypeConfig::Private(ref key) => {
                let mut rng = rand::rngs::OsRng;

                // there is actually no way to build an Options object that
                // doesn't start from 0, but the fact that internally is a range
                // allows it, so I take the length of the interval just in case
                // for the size of the unit vector. There is no difference
                // anyway if the start is zero
                let length = self
                    .options
                    .choice_range()
                    .end
                    .checked_sub(self.options.choice_range().start)?;

                // the Choice was validated already, so this can't overflow
                let choice = choice.as_byte() - self.options.choice_range().start;

                let vote = Vote::new(length.into(), choice.into());
                let (encrypted_vote, proof) = chain_vote::encrypt_vote(&mut rng, key, vote);

                Payload::Private {
                    encrypted_vote,
                    proof,
                }
            }
        };

        let cast = VoteCast::new(self.vote_plan_id.clone(), self.index, payload);

        Some(cast)
    }
}
