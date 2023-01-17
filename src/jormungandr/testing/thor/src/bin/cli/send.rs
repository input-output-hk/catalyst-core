use crate::cli::command::Error;
use chain_addr::AddressReadable;
use clap::Parser;
use thor::cli::CliController;

#[derive(Parser, Debug)]
pub struct SendCommand {
    // pin
    #[clap(long, short)]
    pub wait: bool,

    #[clap(subcommand)] // Note that we mark a field as a subcommand
    cmd: SendSubCommand,
}

impl SendCommand {
    pub fn exec(self, contoller: CliController) -> Result<(), Error> {
        match self.cmd {
            SendSubCommand::Tx(send_tx) => send_tx.exec(contoller, self.wait),
        }
    }
}

#[derive(Parser, Debug)]
pub enum SendSubCommand {
    Tx(Tx),
}

#[derive(Parser, Debug)]
pub struct Tx {
    /// address in bech32 format
    #[clap(long)]
    pub address: AddressReadable,

    /// ada to send
    #[clap(long)]
    pub ada: u64,

    // pin
    #[clap(long, short)]
    pub pin: String,
}

impl Tx {
    pub fn exec(self, mut contoller: CliController, wait: bool) -> Result<(), Error> {
        contoller.transaction(&self.pin, wait, self.address.to_address().into(), self.ada)?;
        contoller.save_config().map_err(Into::into)
    }
}
