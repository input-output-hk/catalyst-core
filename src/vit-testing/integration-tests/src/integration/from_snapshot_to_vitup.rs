use crate::common::snapshot::SnapshotServiceStarter;
use crate::common::MainnetWallet;
use assert_fs::TempDir;
use catalyst_toolbox::snapshot::voting_group::RepsVotersAssigner;
use catalyst_toolbox::snapshot::Delegations;
use chain_impl_mockchain::certificate::VotePlan;
use fraction::Fraction;
use hersir::builder::Settings;
use jormungandr_automation::testing::block0::Block0ConfigurationExtension;
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::Initial::Fund;
use jormungandr_lib::interfaces::Initial::Token;
use jormungandr_lib::interfaces::InitialToken;
use jormungandr_lib::interfaces::InitialUTxO;
use jormungandr_lib::interfaces::Value;
use mainnet_tools::db_sync::DbSyncInstance;
use mainnet_tools::network::MainnetNetwork;
use mainnet_tools::voting_tools::VotingToolsMock;
use snapshot_trigger_service::config::ConfigurationBuilder;
use snapshot_trigger_service::config::JobParameters;
use std::collections::HashSet;
use vitup::config::ConfigBuilder;
use vitup::testing::vitup_setup;

const DIRECT_VOTING_GROUP: &str = "direct";
const REP_VOTING_GROUP: &str = "dreps";

#[test]
pub fn cip36_mixed_delegation_should_appear_in_block0() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let discrimination = chain_addr::Discrimination::Production;
    let stake = 10_000;

    let alice_voter = MainnetWallet::new(stake);
    let bob_voter = MainnetWallet::new(stake);
    let clarice_voter = MainnetWallet::new(stake);

    let david_representative = MainnetWallet::new(500);
    let edgar_representative = MainnetWallet::new(1_000);
    let fred_representative = MainnetWallet::new(8_000);

    let mut reps = HashSet::new();
    reps.insert(edgar_representative.catalyst_public_key());
    reps.insert(david_representative.catalyst_public_key());
    reps.insert(fred_representative.catalyst_public_key());

    let mut mainnet_network = MainnetNetwork::default();
    let mut db_sync_instance = DbSyncInstance::default();

    mainnet_network.sync_with(&mut db_sync_instance);

    alice_voter
        .send_direct_voting_registration()
        .to(&mut mainnet_network);
    bob_voter
        .send_voting_registration(Delegations::New(vec![(
            david_representative.catalyst_public_key(),
            1,
        )]))
        .to(&mut mainnet_network);
    clarice_voter
        .send_voting_registration(Delegations::New(vec![
            (david_representative.catalyst_public_key(), 1),
            (edgar_representative.catalyst_public_key(), 1),
        ]))
        .to(&mut mainnet_network);

    let voting_tools =
        VotingToolsMock::default().connect_to_db_sync(&db_sync_instance, &testing_directory);

    let configuration = ConfigurationBuilder::default()
        .with_voting_tools_params(voting_tools.into())
        .with_tmp_result_dir(&testing_directory)
        .build();

    let assigner = RepsVotersAssigner::new_from_repsdb(
        DIRECT_VOTING_GROUP.to_string(),
        REP_VOTING_GROUP.to_string(),
        reps,
    )
    .unwrap();

    let snapshot_service = SnapshotServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(&testing_directory)
        .unwrap();

    let voter_hir = snapshot_service
        .snapshot(
            JobParameters::fund("fund9"),
            450u64,
            Fraction::from(1u64),
            &assigner,
        )
        .to_voter_hir();

    let mut config = ConfigBuilder::default().build();
    config
        .initials
        .block0
        .extend_from_external(voter_hir, discrimination);
    let (controller, _, _) = vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();

    controller
        .settings()
        .assert_has_funds(alice_voter.catalyst_public_key(), stake.into());
    controller
        .settings()
        .assert_has_no_entry(bob_voter.catalyst_public_key());
    controller
        .settings()
        .assert_has_no_entry(clarice_voter.catalyst_public_key());

    controller.settings().assert_has_funds(
        david_representative.catalyst_public_key(),
        (stake + stake / 2).into(),
    );
    controller.settings().assert_has_funds(
        edgar_representative.catalyst_public_key(),
        (stake / 2).into(),
    );
    controller
        .settings()
        .assert_has_no_entry(fred_representative.catalyst_public_key());

    controller
        .settings()
        .assert_has_voting_token_for_direct_voting(alice_voter.catalyst_public_key(), stake.into());
    controller
        .settings()
        .assert_has_voting_token_for_rep_voting(
            david_representative.catalyst_public_key(),
            (stake + stake / 2).into(),
        );
    controller
        .settings()
        .assert_has_voting_token_for_rep_voting(
            edgar_representative.catalyst_public_key(),
            (stake / 2).into(),
        );
}

pub trait SettingsExtensions {
    fn assert_has_funds(&self, identifier: Identifier, funds: Value);
    fn assert_has_voting_token_for_direct_voting(&self, identifier: Identifier, funds: Value);
    fn assert_has_voting_token_for_rep_voting(&self, identifier: Identifier, funds: Value);
    fn assert_has_no_entry(&self, identifier: Identifier);
    fn utxos(&self) -> Vec<InitialUTxO>;
    fn token_with_destination(&self, identifier: Identifier, funds: Value) -> Option<InitialToken>;
    fn tokens(&self) -> Vec<InitialToken>;
    fn vote_plan_with_alias_contains(&self, alias_part: &str) -> Option<VotePlan>;
}

impl SettingsExtensions for Settings {
    fn assert_has_voting_token_for_direct_voting(&self, identifier: Identifier, funds: Value) {
        let vote_plan = self.vote_plan_with_alias_contains("direct").unwrap();
        let voting_token: jormungandr_lib::interfaces::TokenIdentifier =
            vote_plan.voting_token().clone().into();
        assert_eq!(
            voting_token,
            self.token_with_destination(identifier, funds)
                .unwrap()
                .token_id
        )
    }

    fn assert_has_voting_token_for_rep_voting(&self, identifier: Identifier, funds: Value) {
        let vote_plan = self.vote_plan_with_alias_contains("dreps").unwrap();
        let voting_token: jormungandr_lib::interfaces::TokenIdentifier =
            vote_plan.voting_token().clone().into();
        assert_eq!(
            voting_token,
            self.token_with_destination(identifier, funds)
                .unwrap()
                .token_id
        )
    }

    fn vote_plan_with_alias_contains(&self, alias_part: &str) -> Option<VotePlan> {
        let vote_plan = self
            .vote_plans
            .iter()
            .find(|(key, _)| key.alias.contains(alias_part))
            .map(|(_, y)| y)
            .unwrap();
        self.block0
            .vote_plans()
            .iter()
            .cloned()
            .find(|x| x.to_id() == vote_plan.to_id())
    }

    fn assert_has_funds(&self, identifier: Identifier, funds: Value) {
        let address: jormungandr_lib::interfaces::Address = identifier
            .to_address(self.block0.blockchain_configuration.discrimination)
            .into();
        assert!(self
            .utxos()
            .iter()
            .any(|utxo| { utxo.address == address && utxo.value == funds }));
    }

    fn assert_has_no_entry(&self, identifier: Identifier) {
        let address: jormungandr_lib::interfaces::Address = identifier
            .to_address(self.block0.blockchain_configuration.discrimination)
            .into();
        assert!(!self.utxos().iter().any(|utxo| utxo.address == address));
    }

    fn utxos(&self) -> Vec<InitialUTxO> {
        self.block0
            .initial
            .iter()
            .cloned()
            .fold(vec![], |mut utxos, initial| {
                if let Fund(funds) = initial {
                    utxos.extend(funds);
                }
                utxos
            })
    }

    fn tokens(&self) -> Vec<InitialToken> {
        self.block0
            .initial
            .iter()
            .cloned()
            .fold(vec![], |mut tokens, initial| {
                if let Token(token) = initial {
                    tokens.push(token);
                }
                tokens
            })
    }

    fn token_with_destination(&self, identifier: Identifier, funds: Value) -> Option<InitialToken> {
        let address: jormungandr_lib::interfaces::Address = identifier
            .to_address(self.block0.blockchain_configuration.discrimination)
            .into();
        self.tokens().iter().cloned().find(|token| {
            token
                .to
                .iter()
                .any(|dest| dest.address == address && dest.value == funds)
        })
    }
}
