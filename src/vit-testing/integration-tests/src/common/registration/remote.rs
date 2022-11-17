use registration_service::{
    client::{do_registration, rest::RegistrationRestClient},
    config::Configuration,
};

use assert_fs::TempDir;
use iapyx::utils::qr::SecretFromQrCode;
use jormungandr_lib::crypto::account::Identifier;
use mainnet_lib::MainnetWallet;
use registration_service::client::RegistrationResult;
use registration_service::request::Request;

pub struct RemoteRegistrationServiceController {
    configuration: Configuration,
    client: RegistrationRestClient,
}

impl RemoteRegistrationServiceController {
    pub fn new(configuration: Configuration) -> Self {
        Self {
            client: RegistrationRestClient::new(format!(
                "http://127.0.0.1:{}",
                configuration.address().port()
            )),
            configuration,
        }
    }

    pub fn client(&self) -> &RegistrationRestClient {
        &self.client
    }

    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }

    pub fn self_register(&self, wallet: &MainnetWallet, temp_dir: &TempDir) -> RegistrationResult {
        let registration_request = Request {
            payment_skey: wallet.payment_key().to_hex(),
            payment_vkey: wallet.payment_key().to_public().to_hex(),
            stake_skey: wallet.stake_key().to_hex(),
            stake_vkey: wallet.stake_public_key().to_hex(),
            legacy_skey: Some(wallet.catalyst_secret_key().to_bech32().unwrap()),
            delegation_1: None,
            delegation_2: None,
            delegation_3: None,
        };

        println!("{:?}", registration_request);
        do_registration(registration_request, self.client(), temp_dir)
    }

    pub fn register_with_delegation(
        &self,
        wallet: &MainnetWallet,
        delegations: Vec<(Identifier, u32)>,
        temp_dir: &TempDir,
    ) -> RegistrationResult {
        //TODO: change to error
        if delegations.len() > 3 {
            panic!("only 3 delegation registration are supported in testing");
        }

        fn parse_delegation(input: &(Identifier, u32)) -> String {
            format!("{},{}", input.0.clone().to_bech32_str(), input.1)
        }

        let registration_request = Request {
            payment_skey: wallet.payment_key().to_hex(),
            payment_vkey: wallet.payment_key().to_public().to_hex(),
            stake_skey: wallet.stake_key().to_hex(),
            stake_vkey: wallet.stake_public_key().to_hex(),
            legacy_skey: None,
            delegation_1: delegations.get(0).map(parse_delegation).or(None),
            delegation_2: delegations.get(1).map(parse_delegation).or(None),
            delegation_3: delegations.get(2).map(parse_delegation).or(None),
        };

        println!("Request: {:?}", registration_request);
        do_registration(registration_request, self.client(), temp_dir)
    }
}
