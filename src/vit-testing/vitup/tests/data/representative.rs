use assert_fs::TempDir;
use chain_addr::Discrimination;
use chain_crypto::bech32::Bech32;
use chain_impl_mockchain::tokens::identifier::TokenIdentifier as ChainTokenId;
use itertools::Itertools;
use jormungandr_lib::interfaces::TokenIdentifier;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use valgrind::ValgrindClient;
use vit_servicing_station_tests::common::data::parse_funds;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;
use vitup::builders::utils::DeploymentTree;
use vitup::config::Block0Initial;
use vitup::config::Block0Initials;
use vitup::config::Role;
use vitup::config::{ConfigBuilder, VoteBlockchainTime};
use vitup::testing::{spawn_network, vitup_setup};

#[test]
pub fn representative_multiple_vote_plans() {
    let funds_path = PathBuf::from_str("./resources/example/funds.json").unwrap();
    let mut template_generator = ExternalValidVotingTemplateGenerator::new(
        PathBuf::from_str("./resources/example/proposals.json").unwrap(),
        PathBuf::from_str("./resources/example/challenges.json").unwrap(),
        funds_path.clone(),
        PathBuf::from_str("./resources/example/review.json").unwrap(),
    )
    .unwrap();
    let expected_funds = parse_funds(funds_path).unwrap();
    let funds = 1000;
    let mut rnd = rand::rngs::OsRng;
    let alice = thor::Wallet::new_account(&mut rnd, Discrimination::Production);
    let bob = thor::Wallet::new_account(&mut rnd, Discrimination::Production);

    if expected_funds.len() > 1 {
        panic!("more than 1 expected fund is not supported");
    }
    let expected_fund = expected_funds.iter().next().unwrap().clone();

    let testing_directory = TempDir::new().unwrap().into_persistent();

    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 30,
    };

    let config = ConfigBuilder::default()
        .vote_timing(vote_timing.into())
        .fund_id(expected_fund.id)
        .slot_duration_in_seconds(2)
        .proposals_count(template_generator.proposals_count() as u32)
        .challenges_count(template_generator.challenges_count())
        .reviews_count(3)
        .voting_power(expected_fund.threshold.unwrap() as u64)
        .private(true)
        .block0_initials(Block0Initials(vec![
            Block0Initial::External {
                address: alice.address().to_string(),
                funds,
                role: Role::Voter,
            },
            Block0Initial::External {
                address: bob.address().to_string(),
                funds,
                role: Role::Representative,
            },
        ]))
        .build();

    let (mut controller, vit_parameters, network_params) =
        vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();
    let (_nodes, _vit_station, wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    let files_tree = DeploymentTree::new(testing_directory.path());

    let (voter_token, drep_token) = get_expected_tokens(&files_tree);

    let backend_client = ValgrindClient::new(wallet_proxy.address(), Default::default()).unwrap();

    let direct_vote_plans_ids = vote_plans_ids_for_group(&Role::Voter.to_string(), &backend_client);
    let drep_vote_plans_ids =
        vote_plans_ids_for_group(&Role::Representative.to_string(), &backend_client);

    let vote_plan_statuses = backend_client.node_client().vote_plan_statuses().unwrap();

    assert_eq!(
        vec![voter_token.clone()],
        vote_plan_statuses
            .iter()
            .filter(|v| direct_vote_plans_ids.contains(&v.id.to_string()))
            .map(|v| voting_token_to_string(&v.voting_token))
            .unique()
            .collect::<Vec<String>>()
    );
    assert_eq!(
        vec![drep_token.clone()],
        vote_plan_statuses
            .iter()
            .filter(|v| drep_vote_plans_ids.contains(&v.id.to_string()))
            .map(|v| voting_token_to_string(&v.voting_token))
            .unique()
            .collect::<Vec<String>>()
    );

    let alice_state = backend_client
        .node_client()
        .account_state_by_pk(alice.public_key().to_bech32_str())
        .unwrap();
    let bob_state = backend_client
        .node_client()
        .account_state_by_pk(bob.public_key().to_bech32_str())
        .unwrap();

    assert_eq!(
        vec![voter_token],
        alice_state
            .tokens()
            .iter()
            .map(|(t, _)| voting_token_to_string(t))
            .collect::<Vec<String>>()
    );
    assert_eq!(
        vec![drep_token],
        bob_state
            .tokens()
            .iter()
            .map(|(t, _)| voting_token_to_string(t))
            .collect::<Vec<String>>()
    );
}

fn get_expected_tokens(files_tree: &DeploymentTree) -> (String, String) {
    let contents = std::fs::read_to_string(files_tree.voting_token()).unwrap();
    let voting_tokens: Vec<(Role, TokenIdentifier)> = serde_json::from_str(&contents).unwrap();
    let tokens: HashMap<Role, _> = voting_tokens.iter().cloned().map(|(r, t)| (r, t)).collect();
    (
        voting_token_to_string(tokens.get(&Role::Voter).unwrap()),
        voting_token_to_string(tokens.get(&Role::Representative).unwrap()),
    )
}

fn vote_plans_ids_for_group(group: &str, backend_client: &ValgrindClient) -> Vec<String> {
    backend_client
        .vit_client()
        .proposals(group)
        .unwrap()
        .iter()
        .map(|p| p.voteplan.chain_voteplan_id.clone())
        .unique()
        .collect::<Vec<String>>()
}

fn voting_token_to_string(voting_token: &TokenIdentifier) -> String {
    let token: ChainTokenId = voting_token.clone().into();
    hex::encode(token.token_name)
}
