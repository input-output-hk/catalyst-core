use crate::common::registration::RegistrationServiceStarter;
use crate::common::MainnetWallet;
use assert_fs::TempDir;
use mainnet_tools::cardano_cli::CardanoCliMock;
use mainnet_tools::voter_registration::VoterRegistrationMock;
use registration_service::config::{ConfigurationBuilder, NetworkType};
use registration_service::utils::SecretKeyFromQrCode;

#[test]
pub fn direct_registration_flow() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let stake = 10_000;

    let alice = MainnetWallet::new(stake);

    let voter_registration_mock = VoterRegistrationMock::default();
    let cardano_cli_mock = CardanoCliMock::default();

    let configuration = ConfigurationBuilder::default()
        .with_cardano_cli(cardano_cli_mock.path())
        .with_voter_registration(voter_registration_mock.path())
        .with_network(NetworkType::Mainnet)
        .with_tmp_result_dir(&testing_directory)
        .build();

    let registration_service = RegistrationServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(&testing_directory)
        .unwrap();

    let direct_voting_registration = alice.generate_direct_voting_registration(0);
    voter_registration_mock.with_response(direct_voting_registration, &testing_directory);

    let registration_result = registration_service.self_register(&alice, &testing_directory);

    let key_qr_code = registration_result
        .as_legacy_registration()
        .unwrap()
        .qr_code
        .secret_key_from_qr_code();

    assert_eq!(
        alice.catalyst_secret_key().leak_secret().as_ref(),
        key_qr_code.leak_secret().as_ref()
    );
}

#[test]
pub fn delegation_registration_flow() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let stake = 10_000;

    let alice = MainnetWallet::new(stake);
    let bob = MainnetWallet::new(stake);

    let voter_registration_mock = VoterRegistrationMock::default();
    let cardano_cli_mock = CardanoCliMock::default();

    let configuration = ConfigurationBuilder::default()
        .with_cardano_cli(cardano_cli_mock.path())
        .with_voter_registration(voter_registration_mock.path())
        .with_network(NetworkType::Mainnet)
        .with_tmp_result_dir(&testing_directory)
        .build();

    let registration_service = RegistrationServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(&testing_directory)
        .unwrap();

    let delegations_dist = vec![(bob.catalyst_public_key(), 1u32)];

    let delegation_voting_registration =
        alice.generate_delegated_voting_registration(delegations_dist.clone(), 0);
    voter_registration_mock.with_response(delegation_voting_registration, &testing_directory);

    let registration_result = registration_service.delegated_register(
        &alice,
        delegations_dist.clone(),
        &testing_directory,
    );
    let info = registration_result.as_delegation_registration().unwrap();

    info.status.assert_is_finished();
    assert_eq!(
        info.delegations,
        delegations_dist
            .iter()
            .map(|(id, weight)| (id.to_bech32_str(), *weight))
            .collect()
    );
}
