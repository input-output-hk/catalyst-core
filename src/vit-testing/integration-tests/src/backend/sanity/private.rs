use crate::common::iapyx_from_qr;
use crate::common::{vitup_setup, wait_until_folder_contains_all_qrs, Error, Vote};
use assert_fs::TempDir;
use chain_impl_mockchain::block::BlockDate;
use chain_impl_mockchain::key::Hash;
use jormungandr_testing_utils::testing::asserts::VotePlanStatusAssert;
use jormungandr_testing_utils::testing::BlockDateGenerator;
use jormungandr_testing_utils::testing::FragmentSender;
use jormungandr_testing_utils::testing::{node::time, FragmentSenderSetup};
use std::path::Path;
use std::str::FromStr;
use valgrind::Protocol;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::builders::VitBackendSettingsBuilder;
use vitup::config::VoteBlockchainTime;
use vitup::config::{InitialEntry, Initials};
use vitup::scenario::network::setup_network;

#[test]
pub fn private_vote_e2e_flow() -> std::result::Result<(), Error> {
    let endpoint = "127.0.0.1:8080";
    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 60,
    };

    let testing_directory = TempDir::new().unwrap().into_persistent();
    let mut quick_setup = VitBackendSettingsBuilder::new();
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
        .slot_duration_in_seconds(2)
        .proposals_count(1)
        .vote_timing(vote_timing.into())
        .voting_power(8_000)
        .private(true);

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

    let mut committee = controller.wallet("committee_1").unwrap();

    let leader_1 = &nodes[0];
    let wallet_node = &nodes[3];

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("vit_backend/qr-codes");
    wait_until_folder_contains_all_qrs(3, &qr_codes_folder);
    let david_qr_code = Path::new(&qr_codes_folder).join("wallet_david_1234.png");
    let edgar_qr_code = Path::new(&qr_codes_folder).join("wallet_edgar_1234.png");
    let filip_qr_code = Path::new(&qr_codes_folder).join("wallet_filip_1234.png");

    // start mainnet wallets
    let mut david = iapyx_from_qr(&david_qr_code, "1234", &wallet_proxy).unwrap();

    let fund1_vote_plan = controller.vote_plan(&fund_name).unwrap();

    // start voting
    david
        .vote_for(fund1_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    let mut edgar = iapyx_from_qr(&edgar_qr_code, "1234", &wallet_proxy).unwrap();

    edgar
        .vote_for(fund1_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    let mut filip = iapyx_from_qr(&filip_qr_code, "1234", &wallet_proxy).unwrap();

    filip
        .vote_for(fund1_vote_plan.id(), 0, Vote::No as u8)
        .unwrap();

    let target_date = BlockDate {
        epoch: 1,
        slot_id: 5,
    };
    time::wait_for_date(target_date.into(), leader_1.rest());
    let settings = wallet_node.rest().settings().unwrap();
    let block_date_generator = BlockDateGenerator::rolling(
        &settings,
        BlockDate {
            epoch: 1,
            slot_id: 0,
        },
        false,
    );

    let fragment_sender = FragmentSender::new(
        Hash::from_str(&settings.block0_hash).unwrap().into(),
        settings.fees,
        block_date_generator,
        FragmentSenderSetup::resend_3_times(),
    );

    fragment_sender
        .send_encrypted_tally(&mut committee, &fund1_vote_plan.clone().into(), wallet_node)
        .unwrap();

    let target_date = BlockDate {
        epoch: 1,
        slot_id: 30,
    };
    time::wait_for_date(target_date.into(), leader_1.rest());

    let active_vote_plans = leader_1.rest().vote_plan_statuses().unwrap();
    let vote_plan_status = active_vote_plans
        .iter()
        .find(|c_vote_plan| c_vote_plan.id == Hash::from_str(&fund1_vote_plan.id()).unwrap().into())
        .unwrap();

    let shares = {
        match controller
            .settings()
            .vote_plans
            .iter()
            .find(|(key, vote_plan)| key.alias == fund_name)
            .map(|(key, vote_plan)| vote_plan)
            .unwrap()
        {
            VotePlanSettings::Public(_) => panic!("unexpected public voteplan"),
            VotePlanSettings::Private { keys, vote_plan } => keys
                .decrypt_tally(&vote_plan_status.clone().into())
                .unwrap(),
        }
    };

    fragment_sender
        .send_private_vote_tally(
            &mut committee,
            &fund1_vote_plan.clone().into(),
            shares,
            wallet_node,
        )
        .unwrap();

    vote_timing.wait_for_tally_end(leader_1.rest());

    leader_1
        .rest()
        .vote_plan_statuses()
        .unwrap()
        .assert_all_proposals_are_tallied();

    vit_station.shutdown();
    wallet_proxy.shutdown();
    for mut node in nodes {
        node.shutdown()?;
    }
    controller.finalize();
    Ok(())
}
