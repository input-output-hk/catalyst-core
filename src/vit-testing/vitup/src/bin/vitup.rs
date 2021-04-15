use structopt::StructOpt;
use vitup::error::Result;
use vitup::setup::VitCliCommand;

pub fn main() -> Result<()> {
    VitCliCommand::from_args().exec()
}
