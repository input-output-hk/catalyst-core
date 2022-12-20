mod explorer;

use assert_fs::TempDir;
use explorer::{transaction_by_id, TransactionById};
use graphql_client::{GraphQLQuery, Response};
use jormungandr_automation::jormungandr::explorer::configuration::ExplorerParams;
use jormungandr_automation::jormungandr::{
    ExplorerError, JormungandrBootstrapper, JormungandrError, JormungandrProcess,
    NodeConfigBuilder, RestError, StartupError, StartupVerificationMode,
};
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::interfaces::{Log, LogEntry, LogOutput, VotePlanStatus};
use std::path::PathBuf;
use std::str::FromStr;

const JORMUNGANDR_APP: &str = "jormungandr";

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

        let node_config = NodeConfigBuilder::default()
            .with_storage(storage)
            .with_log(Log(LogEntry {
                level: "info".to_string(),
                format: "json".to_string(),
                output: LogOutput::Stdout,
            }))
            .build();

        let inner = JormungandrBootstrapper::default_with_config(node_config)
            .passive()
            .with_block0_hash(Hash::from_str(&genesis_block_hash).unwrap())
            .with_jormungandr(jormungandr_bin.unwrap_or_else(|| PathBuf::from(JORMUNGANDR_APP)))
            .into_starter(temp_dir)
            .unwrap()
            .verify_by(StartupVerificationMode::Log)
            .start()
            .unwrap();

        inner.check_no_errors_in_log()?;
        Ok(Self { inner })
    }

    pub fn active_vote_plans(&self) -> Result<Vec<VotePlanStatus>, Error> {
        Ok(self.inner.rest().vote_plan_statuses()?)
    }

    /// Check that all transactions are present on the main chain of the node
    pub fn check_transactions_on_chain(&self, transactions: Vec<String>) -> Result<(), Error> {
        let tip = self.inner.rest().tip()?.to_string();
        let explorer = self.inner.explorer(ExplorerParams::default())?;
        let explorer = explorer.client();

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
