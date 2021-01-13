use structopt::StructOpt;
use vitup::error::Result;
use vitup::setup::args::VitCliCommand;

fn main() -> Result<()> {
    VitCliCommand::from_args().exec()
}
