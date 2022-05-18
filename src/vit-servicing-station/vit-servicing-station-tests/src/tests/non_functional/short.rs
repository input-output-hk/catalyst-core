use super::MockStatusProvider;
use crate::common::{
    load::{VitRestRequestGenerator, VotingPowerRequestGenerator},
    snapshot::{SnapshotBuilder, SnapshotUpdater},
    startup::quick_start,
};
use assert_fs::TempDir;
use jortestkit::load::{self, ConfigurationBuilder, Monitor};
use std::time::Duration;

#[test]
pub fn update_snapshot_during_the_load_quick() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let snapshot = SnapshotBuilder::default()
        .with_entries_count(10_000)
        .build();

    rest_client.put_snapshot(&snapshot).unwrap();

    let request = VotingPowerRequestGenerator::new(snapshot.clone(), rest_client.clone());
    let config = ConfigurationBuilder::duration(Duration::from_secs(40))
        .thread_no(10)
        .step_delay(Duration::from_millis(500))
        .monitor(Monitor::Progress(100))
        .build();

    let load_run = load::start_background_async(
        request,
        MockStatusProvider,
        config,
        "Vit station snapshot service rest",
    );

    std::thread::sleep(std::time::Duration::from_secs(10));

    let new_snapshot = SnapshotUpdater::from(snapshot)
        .update_voting_power()
        .add_new_arbitrary_voters()
        .build();

    rest_client.put_snapshot(&new_snapshot).unwrap();

    let stats = load_run.wait_for_finish();
    assert!((stats.calculate_passrate() as u32) > 95);
}

#[test]
pub fn rest_snapshot_load_quick() {
    let temp_dir = TempDir::new().unwrap();
    let (server, data) = quick_start(&temp_dir).unwrap();
    let rest_client = server.rest_client_with_token(&data.token_hash());

    let snapshot = SnapshotBuilder::default()
        .with_entries_count(10_000)
        .build();

    let request = VotingPowerRequestGenerator::new(snapshot, rest_client);
    let config = ConfigurationBuilder::duration(Duration::from_secs(40))
        .thread_no(10)
        .step_delay(Duration::from_millis(500))
        .monitor(Monitor::Progress(100))
        .build();
    let stats = load::start_sync(request, config, "Vit station snapshot service rest");
    assert!((stats.calculate_passrate() as u32) > 95);
}

#[test]
pub fn rest_load_quick() {
    let temp_dir = TempDir::new().unwrap();
    let (server, snapshot) = quick_start(&temp_dir).unwrap();

    let rest_client = server.rest_client();

    let request = VitRestRequestGenerator::new(snapshot, rest_client);
    let config = ConfigurationBuilder::duration(Duration::from_secs(40))
        .thread_no(10)
        .step_delay(Duration::from_millis(500))
        .monitor(Monitor::Progress(100))
        .build();
    let stats = load::start_sync(request, config, "Vit station service rest");
    assert!((stats.calculate_passrate() as u32) > 95);
}
