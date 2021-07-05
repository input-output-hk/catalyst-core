use registration_verify_service::client::args::{Error, RegistrationVerifyServiceCliCommand};
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");

    RegistrationVerifyServiceCliCommand::from_args().exec()
}
