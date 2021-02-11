use crate::scenario::vit_station::VitStationControllerError;
use crate::scenario::wallet::WalletProxyError;
use jormungandr_lib::interfaces::FragmentStatus;
use std::time::Duration;

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
        TemplateLoadError(vit_servicing_station_tests::common::data::TemplateLoadError);
        SerdeError(serde_json::Error);
        SerdeYamlError(serde_yaml::Error);
        Block0EncodeError(chain_impl_mockchain::ledger::Error);
        ScenarioError(jormungandr_scenario_tests::scenario::Error);
        GeneralError(jormungandr_scenario_tests::test::Error);
        ImageReadError(image::error::ImageError);
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

        SnapshotIntialReadError {
            description("wrong format for snapshot data"),
            display("wrong format for snapshot data"),
        }
    }
}
