use structopt::StructOpt;
use vitup::error::Result;
use vitup::setup::VitCliCommand;

#[tokio::main]
pub async fn main() -> Result<()> {
    VitCliCommand::from_args().exec().await
}
