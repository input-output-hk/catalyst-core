use iapyx::cli::args::stats::IapyxStatsCommand;
use structopt::StructOpt;

pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");
    IapyxStatsCommand::from_args().exec().unwrap()
}
