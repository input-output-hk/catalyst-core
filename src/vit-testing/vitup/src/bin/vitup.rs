use clap::Parser;
use vitup::cli::VitCliCommand;

pub fn main() -> std::result::Result<(), Box<vitup::error::Error>> {
    VitCliCommand::parse().exec().map_err(Box::new)
}
