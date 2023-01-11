mod cli;

use cli::command::Command;
use clap::Parser;
use thor::cli::CliController;

pub fn main() {
    let controller = CliController::new().unwrap();
    Command::parse().exec(controller).unwrap();
}
