use clap::Parser;

pub mod cli;
pub mod logger;
pub mod service;
pub mod settings;
pub mod db;

#[tokio::main]
async fn main() -> Result<(), cli::Error> {
    cli::Cli::parse().exec().await?;
    Ok(())
}
