use crate::common::{
    load::{VitRestRequestGenerator, VotingPowerRequestGenerator},
    snapshot::SnapshotBuilder,
    startup::quick_start,
};
use assert_fs::TempDir;
use jortestkit::load::{self, ConfigurationBuilder, Monitor};
use std::time::Duration;

#[test]
#[cfg(feature = "soak")]
pub fn rest_snapshot_load_long() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let snapshot = SnapshotBuilder::default()
        .with_entries_count(10_000)
        .build();

    let request = VotingPowerRequestGenerator::new(snapshot, rest_client);
    let config = ConfigurationBuilder::duration(Duration::from_secs(18_000))
        .thread_no(3)
        .step_delay(Duration::from_secs(1))
        .monitor(Monitor::Progress(10_000))
        .build();
    let stats = load::start_sync(request, config, "Vit station snapshot service rest");
    assert!((stats.calculate_passrate() as u32) > 95);
}

#[test]
#[cfg(feature = "soak")]
pub fn rest_load_long() {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let rest_client = server.rest_client();

    let request = VitRestRequestGenerator::new(snapshot, rest_client);
    let config = ConfigurationBuilder::duration(Duration::from_secs(18_000))
        .thread_no(3)
        .step_delay(Duration::from_secs(1))
        .monitor(Monitor::Progress(10_000))
        .build();
    let stats = load::start_sync(request, config, "Vit station service rest");
    assert!((stats.calculate_passrate() as u32) > 95);
}
