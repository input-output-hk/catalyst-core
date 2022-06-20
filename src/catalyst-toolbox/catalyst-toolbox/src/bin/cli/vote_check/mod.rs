use color_eyre::eyre::bail;
use color_eyre::Report;
use structopt::StructOpt;

use catalyst_toolbox::vote_check::CheckNode;

use jormungandr_lib::interfaces::VotePlanStatus;

use std::fs::File;
use std::path::PathBuf;

/// Verify that your votes were correctly tallied.
///
/// Requires Jormungandr to be installed in the system
#[derive(Debug, PartialEq, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct VoteCheck {
    /// Path to folder containing the full blockchain history saved in Jormungandr
    /// storage format.
    #[structopt(short, long, parse(from_os_str))]
    blockchain: PathBuf,
    /// Genesis block hash
    #[structopt(short, long)]
    genesis_block_hash: String,
    /// Ids of the transactions to check
    #[structopt(short, long)]
    transactions: Vec<String>,
    /// Path to the expected results of the election, in Json format as returned by the /vote/active/plans endpoint
    #[structopt(short, long)]
    expected_results: PathBuf,
    /// Path to the Jormungandr binary. If not provided, will look for 'jormungandr' in PATH
    #[structopt(short, long)]
    jormungandr_bin: Option<PathBuf>,
}

impl VoteCheck {
    /// Vote verification follows this plan:
    ///  * Start a new node with the storage containing the full blockchain history to validate
    ///    that all ledger operations.
    ///  * Check that the election results obtained are the same as provided
    ///  * Check that the transactions containing your votes were indeed included in a block
    ///    in the main chain
    ///
    pub fn exec(self) -> Result<(), Report> {
        let node = CheckNode::spawn(
            self.blockchain.clone(),
            self.genesis_block_hash.clone(),
            self.jormungandr_bin,
        )?;

        let expected_results: Vec<VotePlanStatus> =
            serde_json::from_reader(File::open(self.expected_results)?)?;
        let actual_results = node.active_vote_plans()?;

        for vote_plan in expected_results {
            if !actual_results.contains(&vote_plan) {
                let expected = serde_json::to_string_pretty(&vote_plan).unwrap();
                let actual = actual_results
                    .iter()
                    .find(|act| act.id == vote_plan.id)
                    .map(|act| serde_json::to_string_pretty(act).unwrap())
                    .unwrap_or_default();

                bail!("results do not match, expected: {expected:?}, actual: {actual:?}");
            }
        }

        node.check_transactions_on_chain(self.transactions)?;

        println!("Vote(s) correctly validated!");

        Ok(())
    }
}
