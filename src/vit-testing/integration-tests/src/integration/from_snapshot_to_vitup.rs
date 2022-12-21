use crate::common::snapshot::mock;
use crate::common::{CardanoWallet, RepsVoterAssignerSource, SnapshotFilter};
use assert_fs::TempDir;
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
use mainnet_lib::network::wallet_state::{MainnetNetworkBuilder, MainnetWalletStateBuilder};
use snapshot_trigger_service::config::JobParameters;
use vitup::config::ConfigBuilder;
use vitup::testing::vitup_setup;

#[test]
pub fn cip36_mixed_delegation_should_appear_in_block0() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let discrimination = chain_addr::Discrimination::Production;
    let stake = 10_000;

    let alice = CardanoWallet::new(stake);
    let bob = CardanoWallet::new(stake);
    let clarice = CardanoWallet::new(stake);

    let david_representative = CardanoWallet::new(500);
    let edgar_representative = CardanoWallet::new(1_000);
    let fred_representative = CardanoWallet::new(8_000);

    let (db_sync, _node, reps) = MainnetNetworkBuilder::default()
        .with(alice.as_direct_voter())
        .with(bob.as_delegator(vec![(&david_representative, 1)]))
        .with(clarice.as_delegator(vec![(&edgar_representative, 1), (&edgar_representative, 1)]))
        .build();

    let snapshot_result =
        mock::do_snapshot(&db_sync, JobParameters::fund("fund9"), &testing_directory).unwrap();

    let voter_hir = SnapshotFilter::from_snapshot_result(
        &snapshot_result,
        450u64.into(),
        Fraction::from(1u64),
        &reps.into_reps_voter_assigner(),
    )
    .to_voters_hirs();

    let mut config = ConfigBuilder::default().build();
    config
        .initials
        .block0
        .extend_from_external(voter_hir, discrimination);
    let (controller, _, _) = vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();

    controller
        .settings()
        .assert_has_funds(alice.catalyst_public_key(), stake.into());
    controller
        .settings()
        .assert_has_no_entry(bob.catalyst_public_key());
    controller
        .settings()
        .assert_has_no_entry(clarice.catalyst_public_key());

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
        .assert_has_voting_token_for_direct_voting(alice.catalyst_public_key(), stake.into());
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
