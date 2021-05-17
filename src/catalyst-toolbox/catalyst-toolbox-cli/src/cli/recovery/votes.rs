use super::Error;
use catalyst_toolbox_lib::recovery::tally::{deconstruct_account_transaction, VoteFragmentFilter};
use chain_core::property::Deserialize;
use chain_impl_mockchain::block::Block;
use chain_impl_mockchain::fragment::Fragment;
use chain_impl_mockchain::transaction::Transaction;
use chain_impl_mockchain::vote::Payload;
use jcli_lib::utils::{output_file::OutputFile, output_format::OutputFormat};
use jormungandr_lib::interfaces::load_persistent_fragments_logs_from_folder_path;
use serde::Serialize;
use std::collections::HashMap;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct VotesPrintout {
    /// Path to the block0 binary file
    #[structopt(long)]
    block0_path: PathBuf,

    /// Path to the folder containing the log files used for the tally reconstruction
    #[structopt(long)]
    logs_path: PathBuf,

    #[structopt(flatten)]
    output: OutputFile,

    #[structopt(flatten)]
    output_format: OutputFormat,
}

#[derive(Serialize)]
struct VoteCast {
    public_key: String,
    voteplan: String,
    proposal: u8,
    choice: u8,
}

impl From<Transaction<chain_impl_mockchain::certificate::VoteCast>> for VoteCast {
    fn from(transaction: Transaction<chain_impl_mockchain::certificate::VoteCast>) -> Self {
        let (vote_cast, identifier, _) = deconstruct_account_transaction(&transaction.as_slice());
        let choice = if let Payload::Public { choice } = vote_cast.payload() {
            choice
        } else {
            panic!("cannot handle private votes");
        };
        Self {
            public_key: identifier.to_string(),
            voteplan: vote_cast.vote_plan().to_string(),
            proposal: vote_cast.proposal_index(),
            choice: choice.as_byte(),
        }
    }
}

fn group_by_voter(fragments: Vec<Fragment>) -> HashMap<String, Vec<VoteCast>> {
    let mut res = HashMap::new();
    for fragment in fragments {
        if let Fragment::VoteCast(transaction) = fragment {
            let vote_cast = VoteCast::from(transaction);
            res.entry(vote_cast.public_key.clone())
                .or_insert_with(Vec::new)
                .push(vote_cast);
        }
    }
    res
}

impl VotesPrintout {
    pub fn exec(self) -> Result<(), Error> {
        let VotesPrintout {
            block0_path,
            logs_path,
            output,
            output_format,
        } = self;

        let reader = std::fs::File::open(block0_path)?;
        let block0 = Block::deserialize(BufReader::new(reader)).unwrap();

        let (original, to_filter): (Vec<_>, Vec<_>) =
            load_persistent_fragments_logs_from_folder_path(&logs_path)?
                .filter_map(|fragment| match fragment {
                    Ok(persistent) => Some((persistent.fragment.clone(), persistent)),
                    _ => None,
                })
                .unzip();

        let non_filtered_votes = group_by_voter(original);

        let filtered_fragments = VoteFragmentFilter::new(block0, 0..1000, to_filter.into_iter())
            .unwrap()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        let filtered_votes = group_by_voter(filtered_fragments);

        let res = serde_json::json!({
            "original": non_filtered_votes,
            "filtered": filtered_votes,
        });

        let mut out_writer = output.open()?;
        let content = output_format.format_json(res)?;
        out_writer.write_all(content.as_bytes())?;

        Ok(())
    }
}
