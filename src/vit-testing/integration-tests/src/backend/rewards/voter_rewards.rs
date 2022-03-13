use crate::Vote;

use crate::common::funded_proposals;
use assert_fs::TempDir;
use vit_servicing_station_tests::common::data::ProposalTemplate;
use vit_servicing_station_tests::common::data::ValidVotePlanGenerator;
use vit_servicing_station_tests::common::data::ValidVotingTemplateGenerator;
use vit_servicing_station_tests::common::data::{
    ArbitraryValidVotePlanConfig, ChallengeConfig, ChallengeTemplate, ProposalConfig,
};
use vitup::config::{ConfigBuilder, InitialEntry, Initials};
use vitup::testing::vitup_setup;

#[test]
pub fn sanity_block0() {
    let testing_directory = TempDir::new().unwrap().into_persistent();

    let voters_funds: Vec<u64> = vec![1_000_000, 10];

    let config = ConfigBuilder::default()
        .initials(Initials(
            voters_funds
                .iter()
                .cloned()
                .map(InitialEntry::new_random_wallet)
                .collect(),
        ))
        .proposals_count(1)
        .challenges_count(1)
        .voting_power(10)
        .build();

    let mut template = ArbitraryValidVotePlanConfig::default().challenge(
        ChallengeConfig::default()
            .rewards_total(1000)
            .proposers_rewards(1000)
            .proposal(ProposalConfig::default().funds(100)),
    );
    let (_, parameters, _) = vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();
    let mut generator = ValidVotePlanGenerator::new(parameters);
    let snapshot = generator.build(&mut template);

    let mut votes = vec![(
        snapshot.proposals()[0].clone(),
        vec![(Vote::Yes, voters_funds[0]), (Vote::No, voters_funds[1])],
    )];

    let funded_proposal_file = funded_proposals(&testing_directory.into(), snapshot, votes);
}
