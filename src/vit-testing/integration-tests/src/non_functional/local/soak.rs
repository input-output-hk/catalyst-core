use crate::common::load::private_vote_test_scenario;
use vitup::setup::start::quick::QuickVitBackendSettingsBuilder;

#[test]
pub fn soak_test_private_super_optimistic() {
    let no_of_threads = 10;
    let no_of_wallets = 40_000;
    let endpoint = "127.0.0.1:8080";

    let mut quick_setup = QuickVitBackendSettingsBuilder::new();
    quick_setup
        .initials_count(no_of_wallets, "1234")
        .vote_start_epoch(0)
        .tally_start_epoch(30)
        .tally_end_epoch(32)
        .slot_duration_in_seconds(2)
        .slots_in_epoch_count(60)
        .proposals_count(500)
        .voting_power(1_500_000)
        .private(true);

    private_vote_test_scenario(quick_setup, endpoint, no_of_threads, 1);
}
