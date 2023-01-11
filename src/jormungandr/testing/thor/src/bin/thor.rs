mod cli;

use cli::command::Command;
use clap::Parser;
use thor::cli::CliController;

pub fn main() {
    let controller = CliController::new().unwrap();
    Command::from_args().exec(controller).unwrap();
}
