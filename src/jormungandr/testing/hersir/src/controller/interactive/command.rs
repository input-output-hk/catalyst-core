use super::args::{describe, explorer, send, show, spawn};
use clap::Parser;

#[derive(Parser, Debug)]
pub enum InteractiveCommand {
    /// Prints nodes related data, like stats,fragments etc.
    #[clap(subcommand)]
    Show(show::Show),
    /// Spawn leader or passive node (also legacy)
    #[clap(subcommand)]
    Spawn(spawn::Spawn),
    /// Sends Explorer queries
    #[clap(subcommand)]
    Explorer(explorer::Explorer),
    /// Exit interactive mode
    Exit,
    /// Prints wallets, nodes which can be used. Draw topology
    #[clap(subcommand)]
    Describe(describe::Describe),
    /// send fragments
    #[clap(subcommand)]
    Send(send::Send),
}
