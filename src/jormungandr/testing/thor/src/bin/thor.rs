mod cli;

use clap::Parser;
use cli::command::Command;
use thor::cli::CliController;

pub fn main() {
    let controller = CliController::new().unwrap();
    Command::parse().exec(controller).unwrap();
}
