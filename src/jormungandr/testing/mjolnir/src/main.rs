use clap::Parser;
use mjolnir::Mjolnir;
use std::error::Error;

fn main() {
    Mjolnir::parse().exec().unwrap_or_else(report_error)
}

fn report_error(error: Box<dyn Error>) {
    eprintln!("{}", error);
    let mut source = error.source();
    while let Some(sub_error) = source {
        eprintln!("  |-> {}", sub_error);
        source = sub_error.source();
    }
    std::process::exit(1)
}
