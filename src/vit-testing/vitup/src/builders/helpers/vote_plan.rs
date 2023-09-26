use crate::config::{Role, VoteBlockchainTime};
use chain_impl_mockchain::certificate::{Proposal, Proposals, PushProposal};
use chain_impl_mockchain::testing::scenario::template::ProposalDefBuilder;
use chain_impl_mockchain::testing::TestGen;
use hersir::builder::VotePlanKey;
use hersir::config::{CommitteeTemplate, PrivateParameters, VotePlanTemplate};
pub use jormungandr_lib::interfaces::Initial;
use jormungandr_lib::interfaces::{BlockDate, TokenIdentifier};
use std::iter;

use thor::WalletAlias;

pub struct VitVotePlanDefBuilder {
    split_by: usize,
    fund_name: String,
    vote_phases: VoteBlockchainTime,
    committee_wallet: WalletAlias,
    proposals_count: usize,
    options: u8,
    private: bool,
    voting_tokens: Vec<(Role, TokenIdentifier)>,
}

impl Default for VitVotePlanDefBuilder {
    fn default() -> Self {
        Self {
            vote_phases: Default::default(),
            split_by: 255,
            proposals_count: 0,
            fund_name: "undefined".to_string(),
            committee_wallet: "undefined".to_string(),
            options: 0,
            private: false,
            voting_tokens: vec![(Default::default(), TestGen::token_id().into())],
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

    pub fn private(mut self, private: bool) -> Self {
        self.private = private;
        self
    }

    pub fn proposals_count(mut self, proposals_count: usize) -> Self {
        self.proposals_count = proposals_count;
        self
    }

    pub fn committee(mut self, committe_wallet: WalletAlias) -> Self {
        self.committee_wallet = committe_wallet;
        self
    }

    pub fn voting_token(mut self, role: Role, voting_token: TokenIdentifier) -> Self {
        self.voting_tokens = vec![(role, voting_token)];
        self
    }

    pub fn voting_tokens(mut self, voting_tokens: Vec<(Role, TokenIdentifier)>) -> Self {
        self.voting_tokens = voting_tokens;
        self
    }

    pub fn build(self) -> Vec<VotePlanTemplate> {
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
        .take(self.proposals_count)
        .collect::<Vec<ProposalDefBuilder>>()
        .chunks(self.split_by)
        .enumerate()
        .flat_map(|(index, proposal_builders)| {
            let vote_plan_name = {
                if index == 0 {
                    self.fund_name.to_string()
                } else {
                    format!("{}_{}", self.fund_name, index)
                }
            };

            self.voting_tokens
                .iter()
                .zip(std::iter::repeat(vote_plan_name))
                .map(|((role, voting_token), vote_plan_name)| {
                    let vote_plan_key = VotePlanKey {
                        alias: format!("{vote_plan_name}-{role}"),
                        owner_alias: self.committee_wallet.to_string(),
                    };

                    VotePlanTemplate {
                        committees: vec![CommitteeTemplate::Generated {
                            alias: self.committee_wallet.to_string(),
                            member_pk: None,
                            communication_pk: None,
                        }],
                        vote_start: BlockDate::new(self.vote_phases.vote_start, 0),
                        vote_end: BlockDate::new(self.vote_phases.tally_start, 0),
                        committee_end: BlockDate::new(self.vote_phases.tally_end, 0),
                        proposals: proposal_builders.iter().map(|pb| pb.clone().build()).fold(
                            Proposals::new(),
                            |mut acc, p| {
                                let proposal: Proposal = p.into();
                                assert_eq!(acc.push(proposal), PushProposal::Success);
                                acc
                            },
                        ),
                        committee_member_public_keys: vec![],
                        voting_token: voting_token.clone(),
                        vote_plan_key,
                        private: self.private.then_some(PrivateParameters {
                            crs: None,
                            threshold: None,
                        }),
                    }
                })
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
    pub fn external_proposal_ids_are_unique(proposal_count: usize) -> TestResult {
        let vote_plans_defs = VitVotePlanDefBuilder::default()
            .proposals_count(proposal_count)
            .options(3)
            .build();

        let mut uniq = HashSet::new();
        TestResult::from_bool(
            vote_plans_defs
                .into_iter()
                .flat_map(|v| v.proposals.to_vec())
                .map(|p| p.external_id().clone())
                .all(move |x| uniq.insert(x)),
        )
    }
}
