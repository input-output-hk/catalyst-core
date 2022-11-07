use crate::common::iapyx_from_secret_key;
use assert_fs::TempDir;
use chain_addr::Discrimination;
use chain_crypto::Ed25519;
use chain_impl_mockchain::block::BlockDate;
use chain_impl_mockchain::certificate::UpdateProposal;
use chain_impl_mockchain::certificate::UpdateVote;
use chain_impl_mockchain::vote::Choice;
use jormungandr_automation::testing::time;
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::crypto::key::SigningKey;
use jormungandr_lib::interfaces::BlockContentMaxSize;
use jormungandr_lib::interfaces::ConfigParam;
use jormungandr_lib::interfaces::ConfigParams;
use jormungandr_lib::interfaces::FragmentStatus;
use std::str::FromStr;
use thor::BlockDateGenerator;
use thor::FragmentSender;
use thor::Wallet;

use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::VoteBlockchainTime;
use vitup::config::{Block0Initial, Block0Initials, ConfigBuilder};
use vitup::testing::{spawn_network, vitup_setup};

const PIN: &str = "1234";
const ALICE: &str = "ALICE";
const COMMITTEE: &str = "COMMITTEE";
#[test]
pub fn increase_max_block_content_size_during_voting() {
    let old_block_context_max_size = 10_000;
    let new_block_context_max_size = 100_000;

    let testing_directory = TempDir::new().unwrap().into_persistent();
    let batch_size = 1;
    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 5,
        tally_end: 10,
        slots_per_epoch: 30,
    };

    let role = Default::default();
    let config = ConfigBuilder::default()
        .block0_initials(Block0Initials(vec![
            Block0Initial::Wallet {
                name: ALICE.to_string(),
                funds: 10_000,
                pin: PIN.to_string(),
                role,
            },
            Block0Initial::Wallet {
                name: COMMITTEE.to_string(),
                funds: 10_000,
                pin: PIN.to_string(),
                role,
            },
        ]))
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(2)
        .proposals_count(300)
        .block_content_max_size(old_block_context_max_size)
        .voting_power(31_000)
        .private(true)
        .build();

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let (mut controller, vit_parameters, network_params) =
        vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();

    let (nodes, _vit_station, wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    let mut alice =
        iapyx_from_secret_key(testing_directory.path().join(ALICE), &wallet_proxy).unwrap();
    let mut committee = Wallet::import_account(
        testing_directory.path().join(COMMITTEE),
        Some(0u32.into()),
        Discrimination::Production,
    );

    let settings = controller.settings();
    let bft_leader_secrets: Vec<&SigningKey<Ed25519>> = settings
        .nodes
        .iter()
        .filter(|(_id, settings)| settings.secret.bft.is_some())
        .take(3)
        .map(|(_id, settings)| &settings.secret.bft.as_ref().unwrap().signing_key)
        .collect();

    let change_params = ConfigParams::new(vec![ConfigParam::BlockContentMaxSize(
        BlockContentMaxSize::from(new_block_context_max_size),
    )]);
    let update_proposal = UpdateProposal::new(
        change_params.into(),
        bft_leader_secrets[0].identifier().into_public_key().into(),
    );

    let old_settings = nodes[1].rest().settings().unwrap();
    assert_eq!(
        old_settings.block_content_max_size,
        old_block_context_max_size
    );

    let wallet_settings = wallet_proxy.client().settings().unwrap();
    let fragment_sender = FragmentSender::new(
        Hash::from_str(&wallet_settings.block0_hash).unwrap(),
        wallet_settings.fees.clone(),
        BlockDateGenerator::rolling(
            &wallet_settings,
            BlockDate {
                epoch: 1,
                slot_id: 0,
            },
            false,
        ),
        Default::default(),
    );

    let check = fragment_sender
        .send_update_proposal(
            &mut committee,
            &bft_leader_secrets[0].clone().into_secret_key(),
            update_proposal,
            &nodes[1],
        )
        .unwrap();

    for bft_leader_secret in bft_leader_secrets {
        let update_vote = UpdateVote::new(
            *check.fragment_id(),
            bft_leader_secret.identifier().into_public_key().into(),
        );
        fragment_sender
            .send_update_vote(
                &mut committee,
                &bft_leader_secret.clone().into_secret_key(),
                update_vote,
                &nodes[1],
            )
            .unwrap();
    }

    time::wait_for_epoch(4, nodes[1].rest());

    let new_settings = nodes[1].rest().settings().unwrap();
    assert_eq!(
        new_settings.block_content_max_size,
        new_block_context_max_size
    );

    //send batch of votes just to be sure everything is ok
    let proposals = alice.proposals(&role.to_string()).unwrap();
    let votes_data: Vec<(&FullProposalInfo, Choice)> = proposals
        .iter()
        .take(batch_size)
        .map(|proposal| (proposal, Choice::new(0)))
        .collect();

    let fragment_ids = alice
        .votes_batch(votes_data.clone())
        .unwrap()
        .iter()
        .map(|item| item.to_string())
        .collect();

    std::thread::sleep(std::time::Duration::from_secs(30));

    let fragment_statuses = nodes[1].rest().fragments_statuses(fragment_ids).unwrap();
    assert!(fragment_statuses
        .iter()
        .all(|(_, status)| matches!(status, FragmentStatus::InABlock { .. })));
}
