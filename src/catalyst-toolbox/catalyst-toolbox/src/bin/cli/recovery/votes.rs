use catalyst_toolbox::recovery::tally::{
    deconstruct_account_transaction, ValidatedFragment, ValidationError, VoteFragmentFilter,
};
use chain_core::{
    packer::Codec,
    property::{Deserialize, Fragment as _},
};
use chain_impl_mockchain::{
    account::SpendingCounter, block::Block, fragment::Fragment, vote::Payload,
};
use color_eyre::Report;
use jcli_lib::utils::{output_file::OutputFile, output_format::OutputFormat};
use jormungandr_lib::interfaces::load_persistent_fragments_logs_from_folder_path;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use std::iter::IntoIterator;
use std::path::PathBuf;
use structopt::StructOpt;

use super::set_verbosity;

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

    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,
}

#[derive(Serialize)]
struct VoteCast {
    public_key: String,
    voteplan: String,
    fragment_id: String,
    chain_proposal_index: u8,
    choice: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    spending_counter: Option<u32>,
}

fn group_by_voter<I: IntoIterator<Item = (Fragment, Option<SpendingCounter>)>>(
    fragments: I,
) -> HashMap<String, Vec<VoteCast>> {
    let mut res = HashMap::new();
    for (fragment, spending_counter) in fragments.into_iter() {
        if let Fragment::VoteCast(ref transaction) = fragment {
            let transaction_info = deconstruct_account_transaction(&transaction.as_slice())
                .and_then(|(vote_cast, identifier, _)| {
                    if let Payload::Public { choice } = vote_cast.payload().clone() {
                        Ok((vote_cast, identifier, choice))
                    } else {
                        Err(ValidationError::UnsupportedPrivateVotes)
                    }
                });
            match transaction_info {
                Ok((vote_cast, identifier, choice)) => {
                    let vote_cast = VoteCast {
                        fragment_id: fragment.id().to_string(),
                        public_key: identifier.to_string(),
                        voteplan: vote_cast.vote_plan().to_string(),
                        chain_proposal_index: vote_cast.proposal_index(),
                        spending_counter: spending_counter.map(Into::into),
                        choice: choice.as_byte(),
                    };
                    res.entry(vote_cast.public_key.clone())
                        .or_insert_with(Vec::new)
                        .push(vote_cast);
                }
                Err(e) => log::error!("Invalid transaction: {}", e),
            }
        }
    }
    res
}

#[derive(Serialize)]
struct RecoveredVotes {
    original: HashMap<String, Vec<VoteCast>>,
    filtered: HashMap<String, Vec<VoteCast>>,
}

impl VotesPrintout {
    pub fn exec(self) -> Result<(), Report> {
        let VotesPrintout {
            block0_path,
            logs_path,
            output,
            output_format,
            verbose,
        } = self;

        set_verbosity(verbose);

        let reader = std::fs::File::open(block0_path)?;
        let block0 = Block::deserialize(&mut Codec::new(reader)).unwrap();

        let (original, to_filter): (Vec<_>, Vec<_>) =
            load_persistent_fragments_logs_from_folder_path(&logs_path)?
                .filter_map(|fragment| match fragment {
                    Ok(persistent) => Some(((persistent.fragment.clone(), None), persistent)),
                    _ => None,
                })
                .unzip();

        let non_filtered_votes = group_by_voter(original);

        let filtered_fragments: Vec<_> =
            VoteFragmentFilter::new(block0, 0..1000, to_filter.into_iter())
                .unwrap()
                .filter_map(Result::ok)
                .map(
                    |ValidatedFragment {
                         fragment,
                         spending_counter,
                         ..
                     }| (fragment, Some(spending_counter)),
                )
                .collect();

        let filtered_votes = group_by_voter(filtered_fragments);

        let res = RecoveredVotes {
            original: non_filtered_votes,
            filtered: filtered_votes,
        };

        let mut out_writer = output.open()?;
        let content = output_format.format_json(serde_json::to_value(res)?)?;
        out_writer.write_all(content.as_bytes())?;

        Ok(())
    }
}
