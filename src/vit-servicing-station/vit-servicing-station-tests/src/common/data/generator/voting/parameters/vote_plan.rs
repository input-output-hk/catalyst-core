use chain_impl_mockchain::testing::scenario::template::{ProposalDef, VotePlanDef};

pub struct SingleVotePlanParameters {
    vote_plan: VotePlanDef,
    vote_encryption_key: Option<String>,
}

impl SingleVotePlanParameters {
    pub fn proposals(&self) -> Vec<ProposalDef> {
        self.vote_plan.proposals()
    }

    pub fn alias(&self) -> String {
        self.vote_plan.alias()
    }

    pub fn vote_plan(&self) -> VotePlanDef {
        self.vote_plan.clone()
    }

    pub fn vote_encryption_key(&self) -> Option<String> {
        self.vote_encryption_key.clone()
    }

    pub fn set_vote_encryption_key(&mut self, vote_encryption_key: String) {
        self.vote_encryption_key = Some(vote_encryption_key);
    }
}

impl From<VotePlanDef> for SingleVotePlanParameters {
    fn from(vote_plan: VotePlanDef) -> Self {
        Self {
            vote_plan,
            vote_encryption_key: None,
        }
    }
}
