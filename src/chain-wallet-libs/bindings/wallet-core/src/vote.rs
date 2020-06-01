use chain_impl_mockchain::{
    certificate::{VoteCast, VotePlanId},
    vote::{Choice, Options, Payload, PayloadType},
};

pub const VOTE_PLAN_ID_LENGTH: usize = 32;

pub struct Proposal {
    vote_plan_id: VotePlanId,
    payload_type: PayloadType,
    index: u8,
    options: Options,
}

impl Proposal {
    pub fn new(
        vote_plan_id: VotePlanId,
        payload_type: PayloadType,
        index: u8,
        options: Options,
    ) -> Self {
        Self {
            vote_plan_id,
            payload_type,
            index,
            options,
        }
    }

    pub fn vote(&self, choice: Choice) -> Option<VoteCast> {
        if !self.options.validate(choice) {
            return None;
        }

        let payload = match self.payload_type {
            PayloadType::Public => Payload::Public { choice },
        };

        let cast = VoteCast::new(self.vote_plan_id.clone(), self.index, payload);

        Some(cast)
    }
}
