use clap::Parser;

mod cli;
mod logger;
mod service;
mod settings;
mod state;
mod types;

#[tokio::main]
async fn main() -> Result<(), cli::Error> {
    cli::Cli::parse().exec().await?;
    Ok(())
}
