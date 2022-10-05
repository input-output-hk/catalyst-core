use structopt::StructOpt;
use vitup::cli::VitCliCommand;
use vitup::Result;

pub fn main() -> Result<()> {
    VitCliCommand::from_args().exec()
}
