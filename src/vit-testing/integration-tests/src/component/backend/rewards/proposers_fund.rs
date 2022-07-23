use crate::Vote;

use crate::common::funded_proposals;
use assert_fs::TempDir;
use vit_servicing_station_tests::common::data::{
    ArbitraryValidVotePlanConfig, ChallengeConfig, ProposalConfig, Snapshot, ValidVotePlanGenerator,
};
use vitup::config::{Block0Initial, Block0Initials, ConfigBuilder};
use vitup::testing::vitup_setup;

#[test]
pub fn single_proposal_in_single_challenge_got_funded() {
    let testing_directory = TempDir::new().unwrap().into_persistent();

    let voters_funds: Vec<u64> = vec![1_000_000, 10];

    let snapshot = ProposalRewardsTestConfig::default()
        .voter_funds(&voters_funds)
        .minimum_voting_power(10)
        .vote_plan(
            ArbitraryValidVotePlanConfig::default().challenge(
                ChallengeConfig::default()
                    .rewards_total(1000)
                    .proposers_rewards(1000)
                    .proposal(ProposalConfig::default().funds(100)),
            ),
        )
        .build(&testing_directory);

    let votes = vec![(
        snapshot.proposals()[0].clone(),
        vec![(Vote::Yes, voters_funds[0]), (Vote::No, voters_funds[1])],
    )];

    let results = funded_proposals(&testing_directory, &snapshot, votes).unwrap();
    let first_challenge = &snapshot.challenges()[0];
    let first_challenge_results = results.challenge_results(&first_challenge.title).unwrap();
    let first_proposal = &snapshot.proposals()[0];
    assert!(first_challenge_results
        .is_funded(&first_proposal.proposal.proposal_title)
        .unwrap());
}

#[derive(Default)]
pub struct ProposalRewardsTestConfig {
    config_builder: ConfigBuilder,
    template: ArbitraryValidVotePlanConfig,
}

impl ProposalRewardsTestConfig {
    pub fn voter_funds(mut self, voters_funds: &[u64]) -> Self {
        self.config_builder = self.config_builder.block0_initials(Block0Initials(
            voters_funds
                .iter()
                .cloned()
                .map(Block0Initial::new_random_wallet)
                .collect(),
        ));
        self
    }
    pub fn minimum_voting_power(mut self, minimum_voting_power: u64) -> Self {
        self.config_builder = self.config_builder.voting_power(minimum_voting_power);
        self
    }
    pub fn vote_plan(mut self, template: ArbitraryValidVotePlanConfig) -> Self {
        let challenges = template.get_challenges();
        self.config_builder = self
            .config_builder
            .proposals_count(challenges.iter().map(|c| c.proposals_len() as u32).sum())
            .challenges_count(challenges.len());
        self.template = template;

        self
    }

    pub fn build(mut self, testing_directory: &TempDir) -> Snapshot {
        let config = self.config_builder.build();
        let (_, parameters, _) =
            vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();
        let mut generator = ValidVotePlanGenerator::new(parameters);
        generator.build(&mut self.template)
    }
}
