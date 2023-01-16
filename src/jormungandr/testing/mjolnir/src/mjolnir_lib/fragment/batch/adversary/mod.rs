mod all;
mod votes_only;

use crate::mjolnir_lib::MjolnirError;
use clap::Parser;
pub use votes_only::VotesOnly;
#[derive(Parser, Debug)]
pub enum Adversary {
    VotesOnly(votes_only::VotesOnly),
    All(all::AdversaryAll),
}

impl Adversary {
    pub fn exec(&self) -> Result<(), MjolnirError> {
        match self {
            Adversary::VotesOnly(votes_only_command) => votes_only_command.exec(),
            Adversary::All(all_command) => all_command.exec(),
        }
    }
}
