use crate::common::static_data::SnapshotExtensions;
use crate::common::vote_plan_status::VotePlanStatusExtension;
use crate::common::vote_plan_status::VotePlanStatusProvider;
use crate::common::CastedVote;
use crate::Vote;
use assert_fs::TempDir;
use catalyst_toolbox::testing::ProposerRewardsExecutor;
use chain_addr::{Address, AddressReadable, Discrimination, Kind};
use jormungandr_automation::testing::block0;
use jormungandr_lib::crypto::key::Identifier;
use std::path::PathBuf;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_lib::db::models::proposals::Proposal;
use vit_servicing_station_tests::common::data::Snapshot;
use vitup::builders::utils::DeploymentTree;

pub type VotesRegistry = Vec<(FullProposalInfo, Vec<(Vote, u64)>)>;

pub fn funded_proposals(
    testing_directory: &TempDir,
    snapshot: Snapshot,
    registry: VotesRegistry,
) -> PathBuf {
    let deployment = DeploymentTree::from(testing_directory);
    let block0_configuration =
        block0::get_block(deployment.block0_path().to_str().unwrap()).unwrap();

    let proposals_json = testing_directory.path().join("proposals.json");
    let challenges_json = testing_directory.path().join("challenges.json");
    snapshot.dump_proposals(&proposals_json);
    snapshot.dump_challenges(&challenges_json);

    let votes = registry
        .iter()
        .map(|(proposal, votes)| {
            votes
                .iter()
                .map(|vote| CastedVote::from_proposal(proposal, vote.0, vote.1))
                .collect::<Vec<CastedVote>>()
        })
        .flatten()
        .collect();

    let active_vote_plan = block0_configuration.vote_plan_statuses(votes);
    let discrimination = block0_configuration.blockchain_configuration.discrimination;
    let prefix = match discrimination {
        Discrimination::Test => "ta",
        Discrimination::Production => "ca",
    };
    let committee_addresses: Vec<String> = block0_configuration
        .blockchain_configuration
        .committees
        .iter()
        .map(|x| {
            let key = Identifier::from_hex(&x.to_hex()).unwrap();
            let address = AddressReadable::from_address(
                prefix,
                &Address(discrimination, Kind::Account(key.into_public_key())),
            );
            address.to_string()
        })
        .collect();

    let vote_plan_json = testing_directory.path().join("vote_plan.json");
    active_vote_plan.dump(&vote_plan_json);
    let output = testing_directory.path().join("rewards.csv");

    let committee_yaml = testing_directory.path().join("committee.yaml");
    std::fs::write(
        &committee_yaml,
        serde_json::to_string(&committee_addresses).unwrap(),
    )
    .unwrap();

    let rewards = ProposerRewardsExecutor::default()
        .output_file(output.clone())
        .block0_path(deployment.genesis_path())
        .total_stake_threshold(0.01)
        .approval_threshold(0.05)
        .output_format("csv".to_string())
        .proposals_path(proposals_json.to_str().unwrap().to_string())
        .active_voteplan_path(vote_plan_json.to_str().unwrap().to_string())
        .committee_file_path(committee_yaml.to_str().unwrap().to_string())
        .challenges_path(challenges_json.to_str().unwrap().to_string());

    println!("{:?}", rewards);
    rewards.proposers_rewards().unwrap();

    output.to_path_buf()
}
