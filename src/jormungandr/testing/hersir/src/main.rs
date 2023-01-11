use hersir::{args::Args, spawn};
use clap::Parser;

fn main() {
    let args = Args::parse();
    if let Err(e) = spawn::spawn_network(args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
