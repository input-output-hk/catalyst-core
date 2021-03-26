use snapshot_trigger_service::client::args::{Error, TriggerServiceCliCommand};
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");

    TriggerServiceCliCommand::from_args().exec()
}
