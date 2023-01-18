use super::next_id::NextId;
use crate::jcli_lib::rest::{Error, RestArgs};
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Subcommand {
    /// Get block
    Get {
        #[clap(flatten)]
        args: RestArgs,
    },
    /// Get block descendant ID
    #[clap(subcommand)]
    NextId(NextId),
}

impl Subcommand {
    pub fn exec(self, block_id: String) -> Result<(), Error> {
        match self {
            Subcommand::Get { args } => exec_get(block_id, args),
            Subcommand::NextId(next_id) => next_id.exec(block_id),
        }
    }
}

fn exec_get(block_id: String, args: RestArgs) -> Result<(), Error> {
    let response = args
        .client()?
        .get(&["v0", "block", &block_id])
        .execute()?
        .bytes()?;
    println!("{}", hex::encode(&response));
    Ok(())
}
