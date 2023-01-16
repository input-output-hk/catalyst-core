use crate::jcli_lib::rest::Error;
use clap::Parser;

mod next_id;
mod subcommand;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Block {
    /// ID of the block
    block_id: String,

    #[clap(subcommand)]
    subcommand: subcommand::Subcommand,
}

impl Block {
    pub fn exec(self) -> Result<(), Error> {
        self.subcommand.exec(self.block_id)
    }
}
