use clap::Parser;

pub mod cli;
pub mod db;
pub mod logger;
pub mod service;
pub mod settings;

#[tokio::main]
async fn main() -> Result<(), cli::Error> {
    cli::Cli::parse().exec().await?;
    Ok(())
}
