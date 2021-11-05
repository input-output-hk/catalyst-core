use crate::config::VitStartParameters;
use crate::config::VoteBlockchainTime;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use chain_impl_mockchain::testing::scenario::template::{ProposalDefBuilder, VotePlanDefBuilder};
use chain_impl_mockchain::vote::PayloadType;
pub use jormungandr_lib::interfaces::Initial;
use jormungandr_testing_utils::testing::network::WalletAlias;
use std::iter;

pub struct VitVotePlanDefBuilder {
    split_by: usize,
    fund_name: Option<String>,
    vote_phases: VoteBlockchainTime,
    committee_wallet: Option<WalletAlias>,
    options: u8,
    parameters: Option<VitStartParameters>,
}

impl VitVotePlanDefBuilder {
    pub fn new(vote_phases: VoteBlockchainTime) -> Self {
        Self {
            vote_phases,
            split_by: 255,
            fund_name: None,
            committee_wallet: None,
            parameters: None,
            options: 0,
        }
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
        self.fund_name = Some(fund_name);
        self
    }

    pub fn with_committee(mut self, committe_wallet: WalletAlias) -> Self {
        self.committee_wallet = Some(committe_wallet);
        self
    }

    pub fn with_parameters(mut self, parameters: VitStartParameters) -> Self {
        self.parameters = Some(parameters);
        self
    }

    pub fn build(self) -> Vec<VotePlanDef> {
        let fund_name = self.fund_name.as_ref().expect("fund name not defined");
        let parameters = self
            .parameters
            .as_ref()
            .expect("parameters are not defined");

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
        .collect::<Vec<ProposalDefBuilder>>()
        .chunks(self.split_by)
        .into_iter()
        .enumerate()
        .map(|(index, x)| {
            let vote_plan_name = {
                if index == 0 {
                    fund_name.to_string()
                } else {
                    format!("{}_{}", fund_name, index)
                }
            };

            let mut vote_plan_builder = VotePlanDefBuilder::new(&vote_plan_name);
            vote_plan_builder.owner(
                self.committee_wallet
                    .as_ref()
                    .expect("committee wallet not defined"),
            );

            if parameters.private {
                vote_plan_builder.payload_type(PayloadType::Private);
            }

            vote_plan_builder.vote_phases(
                self.vote_phases.vote_start,
                self.vote_phases.tally_start,
                self.vote_phases.tally_end,
            );
            x.to_vec().iter_mut().for_each(|proposal| {
                vote_plan_builder.with_proposal(proposal);
            });
            vote_plan_builder.build()
        })
        .collect()
    }
}
