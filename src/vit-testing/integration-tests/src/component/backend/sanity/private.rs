use crate::common::iapyx_from_qr;
use crate::common::{wait_until_folder_contains_all_qrs, Error};
use crate::Vote;
use assert_fs::TempDir;
use chain_impl_mockchain::block::BlockDate;
use chain_impl_mockchain::key::Hash;
use hersir::builder::VotePlanSettings;
use jormungandr_automation::testing::asserts::VotePlanStatusAssert;
use jormungandr_automation::testing::time;
use std::path::Path;
use std::str::FromStr;
use thor::FragmentSender;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::ConfigBuilder;
use vitup::config::VoteBlockchainTime;
use vitup::config::{Block0Initial, Block0Initials};
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

#[test]
pub fn private_vote_e2e_flow() -> std::result::Result<(), Error> {
    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 60,
    };

    let testing_directory = TempDir::new().unwrap().into_persistent();
    let role = Default::default();
    let config = ConfigBuilder::default()
        .block0_initials(Block0Initials(vec![
            Block0Initial::Wallet {
                name: "david".to_string(),
                funds: 10_000,
                pin: "1234".to_string(),
                role,
            },
            Block0Initial::Wallet {
                name: "edgar".to_string(),
                funds: 10_000,
                pin: "1234".to_string(),
                role,
            },
            Block0Initial::Wallet {
                name: "filip".to_string(),
                funds: 10_000,
                pin: "1234".to_string(),
                role,
            },
        ]))
        .slot_duration_in_seconds(2)
        .proposals_count(1)
        .vote_timing(vote_timing.into())
        .voting_power(8_000)
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

    let mut committee = controller.wallet("committee_1").unwrap();

    let leader_1 = &nodes[0];
    let wallet_node = &nodes[3];

    let mut qr_codes_folder = testing_directory.path().to_path_buf();
    qr_codes_folder.push("qr-codes");
    wait_until_folder_contains_all_qrs(3, &qr_codes_folder);
    let david_qr_code = Path::new(&qr_codes_folder).join("david_1234.png");
    let edgar_qr_code = Path::new(&qr_codes_folder).join("edgar_1234.png");
    let filip_qr_code = Path::new(&qr_codes_folder).join("filip_1234.png");

    let voteplan_alias = format!("{}-{}", config.data.current_fund.fund_info.fund_name, role);

    // start mainnet wallets
    let mut david = iapyx_from_qr(&david_qr_code, "1234", &wallet_proxy).unwrap();
    let fund1_vote_plan = controller.defined_vote_plan(&voteplan_alias).unwrap();

    // start voting
    david
        .vote_for(&fund1_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    let mut edgar = iapyx_from_qr(&edgar_qr_code, "1234", &wallet_proxy).unwrap();

    edgar
        .vote_for(&fund1_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    let mut filip = iapyx_from_qr(&filip_qr_code, "1234", &wallet_proxy).unwrap();

    filip
        .vote_for(&fund1_vote_plan.id(), 0, Vote::No as u8)
        .unwrap();

    let target_date = BlockDate {
        epoch: 1,
        slot_id: 5,
    };
    time::wait_for_date(target_date.into(), leader_1.rest());
    let settings = wallet_node.rest().settings().unwrap();
    let fragment_sender = FragmentSender::from(&settings);

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
            .find(|(key, _)| key.alias == voteplan_alias)
            .map(|(_, vote_plan)| vote_plan)
            .unwrap()
        {
            VotePlanSettings::Public(_) => panic!("unexpected public voteplan"),
            VotePlanSettings::Private { keys, vote_plan: _ } => keys
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

    let vote_plan_statuses = leader_1.rest().vote_plan_statuses().unwrap();

    vote_plan_statuses.assert_proposal_tally(fund1_vote_plan.id(), 0, vec![20_000, 10_000]);

    Ok(())
}
