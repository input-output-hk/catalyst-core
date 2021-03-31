use super::Vote;
use crate::setup::*;
use assert_fs::TempDir;
use chain_impl_mockchain::block::BlockDate;
use chain_impl_mockchain::key::Hash;
use iapyx::Protocol;
use jormungandr_testing_utils::testing::node::time;
use std::path::Path;
use std::str::FromStr;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::{InitialEntry, Initials};
use vitup::scenario::network::setup_network;
use vitup::setup::start::quick::QuickVitBackendSettingsBuilder;

#[test]
pub fn public_vote_e2e_flow() -> std::result::Result<(), crate::Error> {
    let endpoint = "127.0.0.1:8080";
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let mut quick_setup = QuickVitBackendSettingsBuilder::new();
    quick_setup
        .initials(Initials(vec![
            InitialEntry::Wallet {
                name: "david".to_string(),
                funds: 10_000,
                pin: "1234".to_string(),
            },
            InitialEntry::Wallet {
                name: "edgar".to_string(),
                funds: 10_000,
                pin: "1234".to_string(),
            },
            InitialEntry::Wallet {
                name: "filip".to_string(),
                funds: 10_000,
                pin: "1234".to_string(),
            },
        ]))
        .vote_start_epoch(0)
        .tally_start_epoch(1)
        .tally_end_epoch(2)
        .slot_duration_in_seconds(2)
        .slots_in_epoch_count(30)
        .proposals_count(10)
        .voting_power(8_000)
        .private(false);

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();

    let (mut vit_controller, mut controller, vit_parameters, fund_name) =
        vitup_setup(quick_setup, testing_directory.path().to_path_buf());
    let (nodes, vit_station, wallet_proxy) = setup_network(
        &mut controller,
        &mut vit_controller,
        vit_parameters,
        &mut template_generator,
        endpoint.to_string(),
        &Protocol::Http,
        "2.0".to_owned(),
    )
    .unwrap();

    let leader_1 = &nodes[0];
    let wallet_node = &nodes[4];
    let mut committee = controller.wallet("committee_1").unwrap();

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("vit_backend/qr-codes");
    wait_until_folder_contains_all_qrs(3, &qr_codes_folder);

    let david_qr_code = Path::new(&qr_codes_folder).join("wallet_david_1234.png");
    let edgar_qr_code = Path::new(&qr_codes_folder).join("wallet_edgar_1234.png");
    let filip_qr_code = Path::new(&qr_codes_folder).join("wallet_filip_1234.png");

    // start mainnet wallets
    let mut david = vit_controller
        .iapyx_wallet_from_qr(&david_qr_code, "1234", &wallet_proxy)
        .unwrap();
    david.retrieve_funds().unwrap();
    david.convert_and_send().unwrap();

    let fund1_vote_plan = controller.vote_plan(&fund_name).unwrap();

    // start voting
    david
        .vote_for(fund1_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    let mut edgar = vit_controller
        .iapyx_wallet_from_qr(&edgar_qr_code, "1234", &wallet_proxy)
        .unwrap();
    edgar.retrieve_funds().unwrap();
    edgar.convert_and_send().unwrap();

    edgar
        .vote_for(fund1_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    let mut filip = vit_controller
        .iapyx_wallet_from_qr(&filip_qr_code, "1234", &wallet_proxy)
        .unwrap();
    filip.retrieve_funds().unwrap();
    filip.convert_and_send().unwrap();

    filip
        .vote_for(fund1_vote_plan.id(), 0, Vote::No as u8)
        .unwrap();

    let target_date = BlockDate {
        epoch: 1,
        slot_id: 5,
    };
    time::wait_for_date(target_date.into(), leader_1.explorer());

    controller
        .fragment_sender()
        .send_public_vote_tally(&mut committee, &fund1_vote_plan.clone().into(), wallet_node)
        .unwrap();

    time::wait_for_epoch(2, leader_1.explorer());

    let active_vote_plans = leader_1.vote_plans().unwrap();
    let vote_plan_status = active_vote_plans
        .iter()
        .find(|c_vote_plan| c_vote_plan.id == Hash::from_str(&fund1_vote_plan.id()).unwrap().into())
        .unwrap();

    for proposal in vote_plan_status.proposals.iter() {
        assert!(
            proposal.tally.is_some(),
            "Proposal is not tallied {:?}",
            proposal
        );
    }

    vit_station.shutdown();
    wallet_proxy.shutdown();
    for node in nodes {
        node.shutdown()?;
    }
    controller.finalize();
    Ok(())
}
