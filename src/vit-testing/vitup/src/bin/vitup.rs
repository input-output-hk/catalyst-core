use clap::Parser;
use vitup::cli::VitCliCommand;
use vitup::Result;

pub fn main() -> Result<()> {
    VitCliCommand::parse().exec()
}
