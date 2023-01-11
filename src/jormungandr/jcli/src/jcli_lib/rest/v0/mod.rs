pub mod account;
mod block;
mod diagnostic;
mod leaders;
pub mod message;
mod network;
mod node;
mod rewards;
pub mod settings;
mod shutdown;
mod stake;
mod stake_pool;
mod stake_pools;
mod tip;
mod utxo;
mod vote;

use crate::jcli_lib::rest::Error;
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum V0 {
    /// Account operations
    #[clap(subcommand)]
    Account(account::Account),
    /// Block operations
    #[clap(subcommand)]
    Block(block::Block),
    /// Node leaders operations
    #[clap(subcommand)]
    Leaders(leaders::Leaders),
    /// Message sending
    #[clap(subcommand)]
    Message(message::Message),
    /// Network information
    #[clap(subcommand)]
    Network(network::Network),
    /// Node information
    #[clap(subcommand)]
    Node(node::Node),
    /// Node settings
    #[clap(subcommand)]
    Settings(settings::Settings),
    /// Stake information
    #[clap(subcommand)]
    Stake(stake::Stake),
    /// Stake pool operations
    #[clap(subcommand)]
    StakePool(stake_pool::StakePool),
    /// Stake pools operations
    #[clap(subcommand)]
    StakePools(stake_pools::StakePools),
    /// Shutdown node
    #[clap(subcommand)]
    Shutdown(shutdown::Shutdown),
    /// Blockchain tip information
    #[clap(subcommand)]
    Tip(tip::Tip),
    /// UTXO information
    #[clap(subcommand)]
    Utxo(utxo::Utxo),
    /// System diagnostic information
    #[clap(subcommand)]
    Diagnostic(diagnostic::Diagnostic),
    /// Rewards information
    #[clap(subcommand)]
    Rewards(rewards::Rewards),
    /// Vote related operations
    #[clap(subcommand)]
    Vote(vote::Vote),
}

impl V0 {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            V0::Account(account) => account.exec(),
            V0::Block(block) => block.exec(),
            V0::Leaders(leaders) => leaders.exec(),
            V0::Message(message) => message.exec(),
            V0::Network(network) => network.exec(),
            V0::Node(node) => node.exec(),
            V0::Settings(settings) => settings.exec(),
            V0::Stake(stake) => stake.exec(),
            V0::StakePool(stake_pool) => stake_pool.exec(),
            V0::StakePools(stake_pools) => stake_pools.exec(),
            V0::Shutdown(shutdown) => shutdown.exec(),
            V0::Tip(tip) => tip.exec(),
            V0::Utxo(utxo) => utxo.exec(),
            V0::Diagnostic(diagnostic) => diagnostic.exec(),
            V0::Rewards(rewards) => rewards.exec(),
            V0::Vote(vote) => vote.exec(),
        }
    }
}
