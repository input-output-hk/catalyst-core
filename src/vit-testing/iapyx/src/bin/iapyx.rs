mod interactive;

use interactive::{CliController, IapyxCommand};
use clap::Parser;

pub fn main() {
    let controller = CliController::new().unwrap();
    IapyxCommand::from_args().exec(controller).unwrap();
}
