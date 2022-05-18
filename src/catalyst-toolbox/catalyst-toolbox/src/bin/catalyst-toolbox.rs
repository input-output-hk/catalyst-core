use std::error::Error;

use color_eyre::Report;
use structopt::StructOpt as _;

pub mod cli;

fn main() -> Result<(), Report> {
    env_logger::init();
    color_eyre::install()?;
    let result = cli::Cli::from_args().exec();
    if let Err(e) = result {
        report_error(e);
    }
    Ok(())
}

fn report_error(error: Box<dyn Error>) {
    eprintln!("{}", error);
    let mut source = error.source();
    while let Some(sub_error) = source {
        eprintln!("  |-> {}", sub_error);
        source = sub_error.source();
    }
}
