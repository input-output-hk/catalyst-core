use chain_impl_mockchain::{
    certificate::{VoteCast, VotePlanId},
    vote::{Choice, Options, Payload, PayloadType},
};

pub struct VotePlan {
    id: VotePlanId,
    payload_type: PayloadType,
}

pub struct Proposal<'a> {
    vote_plan: &'a VotePlan,
    index: u8,
    options: Options,
}

impl VotePlan {
    pub fn new(id: VotePlanId, payload_type: PayloadType) -> Self {
        Self { id, payload_type }
    }
}

impl<'a> Proposal<'a> {
    pub fn new(vote_plan: &'a VotePlan, index: u8, options: Options) -> Self {
        Self {
            vote_plan,
            index,
            options,
        }
    }

    pub fn vote(&self, choice: Choice) -> Option<VoteCast> {
        if !self.options.validate(choice) {
            return None;
        }

        let payload = match self.vote_plan.payload_type {
            PayloadType::Public => Payload::Public { choice },
        };

        let cast = VoteCast::new(self.vote_plan.id.clone(), self.index, payload);

        Some(cast)
    }
}
