mod adversary;
mod tx_only;

use crate::mjolnir_lib::MjolnirError;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Batch {
    /// Prints nodes related data, like stats,fragments etc.
    TxOnly(tx_only::TxOnly),
    #[clap(subcommand)]
    Adversary(adversary::Adversary),
}

impl Batch {
    pub fn exec(&self) -> Result<(), MjolnirError> {
        match self {
            Batch::TxOnly(tx_only_command) => tx_only_command.exec(),
            Batch::Adversary(adversary_command) => adversary_command.exec(),
        }
    }
}
