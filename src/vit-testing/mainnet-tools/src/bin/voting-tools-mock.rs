use mainnet_tools::voting_tools::VotingToolsCommand;
use structopt::StructOpt;

pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");
    VotingToolsCommand::from_args().exec()
}
