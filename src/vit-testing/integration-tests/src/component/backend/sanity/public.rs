use crate::common::iapyx_from_qr;
use crate::common::{wait_until_folder_contains_all_qrs, Error};
use crate::Vote;
use assert_fs::TempDir;
use chain_impl_mockchain::block::BlockDate;
use jormungandr_automation::testing::asserts::VotePlanStatusAssert;
use jormungandr_automation::testing::time;
use std::path::Path;
use thor::FragmentSender;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::ConfigBuilder;
use vitup::config::VoteBlockchainTime;
use vitup::config::{Block0Initial, Block0Initials};
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

const PIN: &str = "1234";

#[test]
pub fn public_vote_multiple_vote_plans() -> std::result::Result<(), Error> {
    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 30,
    };
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let role = Default::default();
    let config = ConfigBuilder::default()
        .block0_initials(Block0Initials(vec![
            Block0Initial::Wallet {
                name: "david".to_string(),
                funds: 10_000,
                pin: PIN.to_string(),
                role,
            },
            Block0Initial::Wallet {
                name: "edgar".to_string(),
                funds: 10_000,
                pin: PIN.to_string(),
                role,
            },
            Block0Initial::Wallet {
                name: "filip".to_string(),
                funds: 10_000,
                pin: PIN.to_string(),
                role,
            },
        ]))
        .slot_duration_in_seconds(2)
        .vote_timing(vote_timing.into())
        .proposals_count(300)
        .voting_power(8_000)
        .private(false)
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

    let leader_1 = &nodes[0];
    let wallet_node = &nodes[3];
    let mut committee = controller.wallet("committee_1").unwrap();

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("qr-codes");
    wait_until_folder_contains_all_qrs(3, &qr_codes_folder);

    let david_qr_code = Path::new(&qr_codes_folder).join("david_1234.png");
    let edgar_qr_code = Path::new(&qr_codes_folder).join("edgar_1234.png");
    let filip_qr_code = Path::new(&qr_codes_folder).join("filip_1234.png");

    // start mainnet wallets
    let mut david = iapyx_from_qr(&david_qr_code, PIN, &wallet_proxy).unwrap();

    // FIXME: this is a bit brittle, it may be better to parametrize the roles to tokens map in the
    // vitup config, and then filter here by the token identifier
    let vote_plans = controller
        .defined_vote_plans()
        .iter()
        .cloned()
        .filter(|vp| vp.alias().ends_with(&role.to_string()))
        .collect::<Vec<_>>();

    let fund1_vote_plan = &vote_plans[0];
    let fund2_vote_plan = &vote_plans[1];

    let settings = wallet_node.rest().settings().unwrap();

    // start voting
    david
        .vote_for(fund1_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    let mut edgar = iapyx_from_qr(&edgar_qr_code, PIN, &wallet_proxy).unwrap();

    edgar
        .vote_for(fund2_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    let mut filip = iapyx_from_qr(&filip_qr_code, PIN, &wallet_proxy).unwrap();

    filip
        .vote_for(fund1_vote_plan.id(), 0, Vote::No as u8)
        .unwrap();

    let target_date = BlockDate {
        epoch: 1,
        slot_id: 5,
    };
    time::wait_for_date(target_date.into(), leader_1.rest());

    let fragment_sender = FragmentSender::from(&settings);

    fragment_sender
        .send_public_vote_tally(&mut committee, &fund1_vote_plan.clone().into(), wallet_node)
        .unwrap();

    fragment_sender
        .send_public_vote_tally(&mut committee, &fund2_vote_plan.clone().into(), wallet_node)
        .unwrap();

    vote_timing.wait_for_tally_end(leader_1.rest());

    let vote_plans = leader_1.rest().vote_plan_statuses().unwrap();
    vote_plans.assert_proposal_tally(fund1_vote_plan.id(), 0, vec![10_000, 10_000]);
    vote_plans.assert_proposal_tally(fund2_vote_plan.id(), 0, vec![10_000, 0]);
    Ok(())
}
