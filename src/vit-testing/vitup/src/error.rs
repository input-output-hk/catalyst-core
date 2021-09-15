use crate::scenario::vit_station::VitStationControllerError;
use crate::scenario::wallet::WalletProxyError;
use jormungandr_lib::interfaces::Block0ConfigurationError;
use jormungandr_lib::interfaces::FragmentStatus;
use std::time::Duration;
use vit_servicing_station_tests::common::startup::server::ServerBootstrapperError;

error_chain! {

    foreign_links {
        Interactive(jortestkit::console::InteractiveCommandError);
        IoError(std::io::Error);
        Node(jormungandr_scenario_tests::node::Error);
        Wallet(jormungandr_testing_utils::wallet::WalletError);
        FragmentSender(jormungandr_testing_utils::testing::FragmentSenderError);
        FragmentVerifier(jormungandr_testing_utils::testing::FragmentVerifierError);
        VerificationFailed(jormungandr_testing_utils::testing::VerificationError);
        MonitorResourcesError(jormungandr_testing_utils::testing::ConsumptionBenchmarkError);
        ExplorerError(jormungandr_testing_utils::testing::node::ExplorerError);
        VitStationControllerError(VitStationControllerError);
        WalletProxyError(WalletProxyError);
        TemplateLoadError(vit_servicing_station_tests::common::data::TemplateLoad);
        SerdeError(serde_json::Error);
        SerdeYamlError(serde_yaml::Error);
        Block0EncodeError(chain_impl_mockchain::ledger::Error);
        ScenarioError(jormungandr_scenario_tests::scenario::Error);
        GeneralError(jormungandr_scenario_tests::test::Error);
        ImageReadError(image::error::ImageError);
        MockError(crate::mock::Error);
        ClientRestError(crate::client::rest::Error);
        WalletBackendError(iapyx::WalletBackendError);
        Block0ConfigurationError(Block0ConfigurationError);
        VitServerBootstrapperError(ServerBootstrapperError);
        VitRestError(vit_servicing_station_tests::common::clients::RestError);
        ChainAddressError(chain_addr::Error);
        ChainBech32Error(chain_crypto::bech32::Error);
        GlobError(glob::GlobError);
        ControllerError(iapyx::ControllerError);
    }

    errors {
        SyncTimeoutOccurred(info: String, timeout: Duration) {
            description("synchronization for nodes has failed"),
            display("synchronization for nodes has failed. {}. Timeout was: {} s", info, timeout.as_secs()),
        }

        AssertionFailed(info: String) {
            description("assertion has failed"),
            display("{}", info),
        }
        TransactionNotInBlock(node: String, status: FragmentStatus) {
            description("transaction not in block"),
            display("transaction should be 'In Block'. status: {:?}, node: {}", status, node),
        }

        ProxyNotFound(alias: String) {
            description("proxy not found"),
            display("proxy with alias: {} not found", alias),
        }

        EnvironmentIsDown {
            description("environment is down"),
            display("environment is down"),
        }

        SnapshotIntialReadError {
            description("wrong format for snapshot data"),
            display("wrong format for snapshot data"),
        }

        NoChallengeIdFound(proposal_id: String) {
            description("no challenge id found"),
            display("no challenge id found for proposal {}", proposal_id),
        }
    }
}
