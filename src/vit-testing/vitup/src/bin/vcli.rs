use clap::Parser;
use vitup::client::args::VitupClientCommand;

#[tokio::main]
pub async fn main() -> std::result::Result<(), Box<vitup::error::Error>> {
    VitupClientCommand::parse().exec().map_err(Box::new)
}
