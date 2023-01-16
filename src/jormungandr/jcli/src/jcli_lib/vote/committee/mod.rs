mod communication_key;
mod member_key;

use super::Error;
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Committee {
    /// commands for managing committee member communication keys
    #[clap(subcommand)]
    CommunicationKey(communication_key::CommunicationKey),
    /// commands for managing committee member stake keys
    #[clap(subcommand)]
    MemberKey(member_key::MemberKey),
}

impl Committee {
    pub fn exec(self) -> Result<(), super::Error> {
        match self {
            Committee::CommunicationKey(args) => args.exec(),
            Committee::MemberKey(args) => args.exec(),
        }
    }
}
