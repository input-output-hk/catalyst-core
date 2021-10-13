mod explorer;

use assert_fs::{fixture::PathChild, TempDir};
use explorer::{transaction_by_id, TransactionById};
use graphql_client::{GraphQLQuery, Response};
use jormungandr_lib::interfaces::{Log, LogEntry, LogOutput, VotePlanStatus};
use jormungandr_testing_utils::testing::jormungandr::{
    JormungandrError, JormungandrProcess, Starter, StartupError, StartupVerificationMode,
};
use jormungandr_testing_utils::testing::{
    node::{ExplorerError, RestError},
    Block0ConfigurationBuilder, JormungandrParams, NodeConfigBuilder,
};
use std::path::PathBuf;
use std::time::Duration;

const JORMUNGANDR_APP: &str = "jormungandr";
const JORMUNGANDR_LOG_FILE: &str = "node.log";
const JORMUNGANDR_CONFIG_FILE: &str = "node_config.yaml";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Wrapper that exposes the functionalities of the node
/// used for this application
pub struct CheckNode {
    inner: JormungandrProcess,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while reading results from file")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Rest(#[from] RestError),
    #[error(transparent)]
    Explorer(#[from] ExplorerError),
    #[error(transparent)]
    NodeStartup(#[from] StartupError),
    #[error(transparent)]
    ErrorInLogs(#[from] JormungandrError),
    #[error("The transaction with id {0} was not found in the main chain.")]
    TransactionNotOnChain(String),
    #[error(
        "The results of the election are not as expected (expected: {expected}, found: {actual})"
    )]
    ResultsDoNotMatch { expected: String, actual: String },
}

impl CheckNode {
    pub fn spawn(
        storage: PathBuf,
        genesis_block_hash: String,
        jormungandr_bin: Option<PathBuf>,
    ) -> Result<Self, Error> {
        // FIXME: we are using test tools which are not always keen to keep
        // stdout clean of unwanted output.
        // This guard redirects stdout to null untils it's dropped
        let _stdout_mute = gag::Gag::stdout().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let node_config = NodeConfigBuilder::new()
            .with_explorer()
            .with_storage(storage)
            .with_log(Log(LogEntry {
                level: "info".to_string(),
                format: "json".to_string(),
                output: LogOutput::Stdout,
            }))
            .build();

        let config = JormungandrParams::new(
            node_config,
            temp_dir.child(JORMUNGANDR_CONFIG_FILE).path(),
            String::new(),
            genesis_block_hash,
            PathBuf::new(), // passive node with no secrets
            Block0ConfigurationBuilder::new().build(),
            false,
            temp_dir.child(JORMUNGANDR_LOG_FILE).path(),
        );

        config.write_node_config();

        let inner = Starter::new()
            .jormungandr_app(jormungandr_bin.unwrap_or_else(|| PathBuf::from(JORMUNGANDR_APP)))
            .verify_by(StartupVerificationMode::Log)
            .timeout(DEFAULT_TIMEOUT)
            .verbose(false)
            .config(config)
            .temp_dir(temp_dir)
            .passive()
            .start()?;
        inner.check_no_errors_in_log()?;
        Ok(Self { inner })
    }

    pub fn active_vote_plans(&self) -> Result<Vec<VotePlanStatus>, Error> {
        Ok(self.inner.rest().vote_plan_statuses()?)
    }

    /// Check that all transactions are present on the main chain of the node
    pub fn check_transactions_on_chain(&self, transactions: Vec<String>) -> Result<(), Error> {
        let tip = self.inner.rest().tip()?.to_string();
        let mut explorer = self.inner.explorer();
        explorer.disable_logs();

        for id in transactions {
            let res: Response<transaction_by_id::ResponseData> = explorer
                .run(TransactionById::build_query(transaction_by_id::Variables {
                    id: id.clone(),
                }))?
                .json()
                .map_err(ExplorerError::ReqwestError)?;

            if let Some(data) = res.data {
                let mut branch_ids = data
                    .transaction
                    .blocks
                    .into_iter()
                    .flat_map(|block| block.branches)
                    .map(|branch| branch.id);
                if !branch_ids.any(|branch| branch == tip) {
                    return Err(Error::TransactionNotOnChain(id));
                }
            } else {
                return Err(Error::TransactionNotOnChain(id));
            }
        }
        Ok(())
    }
}
