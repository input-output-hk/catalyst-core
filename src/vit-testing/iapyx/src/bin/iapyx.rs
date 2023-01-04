mod interactive;

use interactive::{CliController, IapyxCommand, IapyxCommandError};
use std::error::Error;
use structopt::StructOpt;

pub fn main() {
    exec().unwrap_or_else(report_error)
}

fn exec() -> Result<(), IapyxCommandError> {
    let controller = CliController::new()?;
    IapyxCommand::from_args().exec(controller)
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
