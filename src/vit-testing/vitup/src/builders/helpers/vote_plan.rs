use crate::config::VoteBlockchainTime;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use chain_impl_mockchain::testing::scenario::template::{ProposalDefBuilder, VotePlanDefBuilder};
use chain_impl_mockchain::testing::TestGen;
use chain_impl_mockchain::vote::PayloadType;
pub use jormungandr_lib::interfaces::Initial;
use jormungandr_lib::interfaces::TokenIdentifier;
use std::iter;
use thor::WalletAlias;

pub struct VitVotePlanDefBuilder {
    split_by: usize,
    fund_name: String,
    vote_phases: VoteBlockchainTime,
    committee_wallet: WalletAlias,
    proposals_count: usize,
    options: u8,
    parameters: VitStartParameters,
    voting_token: TokenIdentifier,
}

impl Default for VitVotePlanDefBuilder {
    fn default() -> Self {
        Self {
            vote_phases: Default::default(),
            split_by: 255,
            proposals_count: 0,
            fund_name: "undefined".to_string(),
            committee_wallet: "undefined".to_string(),
            parameters: Default::default(),
            options: 0,
            voting_token: TestGen::token_id().into(),
        }
    }
}

impl VitVotePlanDefBuilder {
    pub fn vote_phases(mut self, vote_phases: VoteBlockchainTime) -> Self {
        self.vote_phases = vote_phases;
        self
    }

    pub fn options(mut self, options: u8) -> Self {
        self.options = options;
        self
    }

    pub fn split_by(mut self, split_by: usize) -> Self {
        self.split_by = split_by;
        self
    }

    pub fn fund_name(mut self, fund_name: String) -> Self {
        self.fund_name = fund_name;
        self
    }

    pub fn with_committee(mut self, committe_wallet: WalletAlias) -> Self {
        self.committee_wallet = committe_wallet;
        self
    }

    pub fn proposals_count(mut self, proposals_count: usize) -> Self {
        self.proposals_count = proposals_count;
        self
    }

    pub fn committee(mut self, committe_wallet: WalletAlias) -> Self {
        self.committee_wallet = Some(committe_wallet);
        self
    }

    pub fn with_parameters(mut self, parameters: VitStartParameters) -> Self {
        self.parameters = parameters;
        self
    }

    pub fn voting_token(mut self, voting_token: TokenIdentifier) -> Self {
        self.voting_token = voting_token;
        self
    }

    pub fn build(self) -> Vec<VotePlanDef> {
        iter::from_fn(|| {
            Some(
                ProposalDefBuilder::new(
                    chain_impl_mockchain::testing::VoteTestGen::external_proposal_id(),
                )
                .options(self.options)
                .action_off_chain()
                .clone(),
            )
        })
        .take(parameters.proposals as usize)
        .take(self.parameters.proposals as usize)
        .collect::<Vec<ProposalDefBuilder>>()
        .chunks(self.split_by)
        .into_iter()
        .enumerate()
        .map(|(index, x)| {
            let vote_plan_name = {
                if index == 0 {
                    self.fund_name.to_string()
                } else {
                    format!("{}_{}", self.fund_name, index)
                }
            };

            let mut vote_plan_builder = VotePlanDefBuilder::new(&vote_plan_name);
            vote_plan_builder
                .voting_token(self.voting_token.clone().into())
                .owner(&self.committee_wallet)
                .vote_phases(
                    self.vote_phases.vote_start,
                    self.vote_phases.tally_start,
                    self.vote_phases.tally_end,
                );

            if self.parameters.private {
                vote_plan_builder.payload_type(PayloadType::Private);
            }

            x.to_vec().iter_mut().for_each(|proposal| {
                vote_plan_builder.with_proposal(proposal);
            });
            vote_plan_builder.build()
        })
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use std::collections::HashSet;

    #[quickcheck]
    pub fn external_proposal_ids_are_unique(proposal_count: u32) -> TestResult {
        let params = VitStartParameters {
            proposals: proposal_count,
            ..Default::default()
        };

        let vote_plans_defs = VitVotePlanDefBuilder::default()
            .fund_name("fundX".to_string())
            .with_committee("fake".to_string())
            .with_parameters(params)
            .build();

        let mut uniq = HashSet::new();
        TestResult::from_bool(
            vote_plans_defs
                .into_iter()
                .flat_map(|v| v.proposals())
                .into_iter()
                .map(|p| p.id())
                .all(move |x| uniq.insert(x)),
        )
    }
}
