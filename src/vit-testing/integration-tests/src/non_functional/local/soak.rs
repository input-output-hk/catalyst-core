use crate::common::load::private_vote_test_scenario;
use vitup::config::{ConfigBuilder, VoteBlockchainTime};

#[test]
pub fn soak_test_private_super_optimistic() {
    let no_of_threads = 10;
    let no_of_wallets = 40_000;
    let endpoint = "127.0.0.1:8080";

    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 30,
        tally_end: 32,
        slots_per_epoch: 60,
    };

    let config = ConfigBuilder::default()
        .initials_count(no_of_wallets, "1234")
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(2)
        .proposals_count(500)
        .voting_power(1_500_000)
        .private(true)
        .build();

    private_vote_test_scenario(config, endpoint, no_of_threads, 1);
}
