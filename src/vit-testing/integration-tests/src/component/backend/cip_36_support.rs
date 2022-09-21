use crate::common::snapshot::do_snapshot;
use crate::common::snapshot_filter::SnapshotFilterSource;
use crate::common::RepsVoterAssignerSource;
use assert_fs::TempDir;
use chain_addr::Discrimination;
use fraction::Fraction;
use snapshot_trigger_service::config::JobParameters;
use std::collections::HashSet;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::Block0Initials;
use vitup::config::ConfigBuilder;
use vitup::config::SnapshotInitials;
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

#[test]
pub fn cip_36_support() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let voting_threshold = 1;
    let tag = None;

    let job_param = JobParameters {
        slot_no: None,
        tag: tag.clone(),
    };

    let snapshot_result = do_snapshot(job_param).unwrap();
    let reps = HashSet::new();

    let snapshot_filter = snapshot_result.filter(
        voting_threshold.into(),
        Fraction::new(1u64, 3u64),
        &reps.into_reps_voter_assigner(),
    );

    let config = ConfigBuilder::default()
        .voting_power(voting_threshold)
        .block0_initials(Block0Initials::new_from_external(
            snapshot_filter.to_voters_hirs(),
            Discrimination::Production,
        ))
        .snapshot_initials(SnapshotInitials::from_voters_hir(
            snapshot_filter.to_voters_hirs(),
            tag.unwrap_or_else(|| "".to_string()),
        ))
        .build();

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();

    let (mut controller, vit_parameters, network_params) =
        vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();
    let (_nodes, _vit_station, _wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(3600))
}
