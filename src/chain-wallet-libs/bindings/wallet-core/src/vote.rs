use chain_impl_mockchain::{
    certificate::{VoteCast, VotePlanId},
    vote::{Choice, Options, Payload, PayloadType},
};

pub struct VotePlan {
    id: VotePlanId,
    payload_type: PayloadType,
}

pub struct Proposal {
    index: u8,
    options: Options,
}

impl VotePlan {
    pub fn new(id: VotePlanId, payload_type: PayloadType) -> Self {
        Self { id, payload_type }
    }
}

impl Proposal {
    pub fn new(index: u8, options: Options) -> Self {
        Self { index, options }
    }

    pub fn vote(&self, vote_plan: &VotePlan, choice: Choice) -> Option<VoteCast> {
        if !self.options.validate(choice) {
            return None;
        }

        let payload = match vote_plan.payload_type {
            PayloadType::Public => Payload::Public { choice },
        };

        let cast = VoteCast::new(vote_plan.id.clone(), self.index, payload);

        Some(cast)
    }
}
