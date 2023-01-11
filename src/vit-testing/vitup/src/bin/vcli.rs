use clap::Parser;
use vitup::client::args::VitupClientCommand;
use vitup::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    VitupClientCommand::parse().exec()
}
