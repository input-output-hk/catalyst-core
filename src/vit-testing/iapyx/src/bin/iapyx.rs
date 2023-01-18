mod interactive;

use clap::Parser;
use interactive::{CliController, IapyxCommand, IapyxCommandError};
use std::error::Error;

pub fn main() {
    exec().unwrap_or_else(report_error)
}

fn exec() -> Result<(), IapyxCommandError> {
    let controller = CliController::new()?;
    IapyxCommand::parse().exec(controller)
}

fn report_error(error: IapyxCommandError) {
    eprintln!("{}", error);
    let mut source = error.source();
    while let Some(sub_error) = source {
        eprintln!("  |-> {}", sub_error);
        source = sub_error.source();
    }
    std::process::exit(1)
}
