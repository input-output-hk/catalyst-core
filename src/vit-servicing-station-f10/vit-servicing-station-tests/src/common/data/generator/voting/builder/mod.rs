mod challenge;
mod proposal;

use crate::common::data::ArbitraryValidVotingTemplateGenerator;
use crate::common::data::ChallengeTemplate;
use crate::common::data::FundTemplate;
use crate::common::data::ProposalTemplate;
use crate::common::data::ReviewTemplate;
use crate::common::data::ValidVotingTemplateGenerator;
pub use challenge::ChallengeConfig;
pub use proposal::ProposalConfig;

#[derive(Clone, Default)]
pub struct ArbitraryValidVotePlanConfig {
    template_generator: ArbitraryValidVotingTemplateGenerator,
    challenges: Vec<ChallengeConfig>,
}

impl ArbitraryValidVotePlanConfig {
    pub fn challenges(mut self, challenges: Vec<ChallengeConfig>) -> Self {
        for challenge in challenges.into_iter() {
            self = self.challenge(challenge);
        }
        self
    }

    pub fn get_challenges(&self) -> &[ChallengeConfig] {
        &self.challenges
    }

    pub fn challenge(mut self, mut challenge: ChallengeConfig) -> Self {
        challenge
            .proposals
            .iter_mut()
            .enumerate()
            .for_each(|(i, mut p)| {
                p.challenge_id = Some(i);
            });
        self.challenges.push(challenge);
        self
    }

    pub fn pop_proposal(&mut self) -> ProposalConfig {
        for challenge in self.challenges.iter_mut() {
            if let Some(proposal) = challenge.proposals.pop_front() {
                return proposal;
            }
        }
        panic!("no more proposals");
    }
}

impl ValidVotingTemplateGenerator for ArbitraryValidVotePlanConfig {
    fn next_proposal(&mut self) -> ProposalTemplate {
        let proposals_builder = self.pop_proposal();
        let challenge = self
            .template_generator
            .challenges
            .get(
                proposals_builder
                    .challenge_id
                    .expect("internal error: no challenge id set for proposal"),
            )
            .unwrap()
            .clone();

        let funds = proposals_builder
            .funds
            .unwrap_or_else(|| self.template_generator.proposal_fund());
        let proposal_template = self.template_generator.proposal(challenge, funds);
        self.template_generator
            .proposals
            .push(proposal_template.clone());
        proposal_template
    }

    fn next_challenge(&mut self) -> ChallengeTemplate {
        let challenge_builder = self
            .challenges
            .get((self.template_generator.next_challenge_id - 1) as usize)
            .expect("no more challenges");
        let mut challenge = self.template_generator.next_challenge();
        if let Some(rewards_total) = challenge_builder.rewards_total {
            challenge.rewards_total = rewards_total.to_string();
        }
        if let Some(proposers_rewards) = challenge_builder.proposers_rewards {
            challenge.proposers_rewards = proposers_rewards.to_string();
        }
        challenge
    }

    fn next_fund(&mut self) -> FundTemplate {
        self.template_generator.next_fund()
    }

    fn next_review(&mut self) -> ReviewTemplate {
        self.template_generator.next_review()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::data::ArbitraryGenerator;
    use crate::common::data::ValidVotePlanGenerator;
    use chain_impl_mockchain::testing::scenario::template::ProposalDefBuilder;
    use chain_impl_mockchain::testing::scenario::template::VotePlanDefBuilder;
    use fake::faker::name::en::Name;
    use fake::Fake;

    #[test]
    pub fn valid_vote_plan_template_builder() {
        let mut vote_plan_parameters = ArbitraryGenerator::default().valid_vote_plan_parameters();

        let mut vote_plan_builder = VotePlanDefBuilder::new("fund_x");
        vote_plan_builder.owner(&Name().fake::<String>());
        vote_plan_builder.vote_phases(1, 2, 3);

        let mut proposal_builder = ProposalDefBuilder::new(
            chain_impl_mockchain::testing::VoteTestGen::external_proposal_id(),
        );
        proposal_builder.options(2);
        proposal_builder.action_off_chain();
        vote_plan_builder.with_proposal(&mut proposal_builder);
        vote_plan_parameters.current_fund.vote_plans = vec![vote_plan_builder.build().into()];
        vote_plan_parameters.current_fund.challenges_count = 1;

        let mut template = ArbitraryValidVotePlanConfig::default().challenge(
            ChallengeConfig::default()
                .rewards_total(1000)
                .proposers_rewards(1000)
                .proposal(ProposalConfig::default().funds(100)),
        );
        let mut generator = ValidVotePlanGenerator::new(vote_plan_parameters);
        let snapshot = generator.build(&mut template);

        println!("{:?}", snapshot);
    }
}
