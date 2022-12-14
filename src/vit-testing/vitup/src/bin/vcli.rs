use structopt::StructOpt;
use vitup::client::args::VitupClientCommand;
use vitup::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    VitupClientCommand::from_args().exec()
}
