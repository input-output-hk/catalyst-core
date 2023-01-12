mod interactive;

use clap::Parser;
use interactive::{CliController, IapyxCommand};

pub fn main() {
    let controller = CliController::new().unwrap();
    IapyxCommand::parse().exec(controller).unwrap();
}
