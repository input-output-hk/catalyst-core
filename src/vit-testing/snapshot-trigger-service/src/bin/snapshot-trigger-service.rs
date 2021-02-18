use snapshot_trigger_service::{Error, TriggerServiceCommand};
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");

    TriggerServiceCommand::from_args().exec()
}
