pub use registration_service::{
    args::{Error, RegistrationServiceCommand},
    context::Context,
    utils::*,
};
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");

    RegistrationServiceCommand::from_args().exec()
}
