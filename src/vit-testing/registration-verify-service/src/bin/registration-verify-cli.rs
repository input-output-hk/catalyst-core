use registration_service::client::args::{Error, RegistrationServiceCliCommand};
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");

    RegistrationServiceCliCommand::from_args().exec()
}
