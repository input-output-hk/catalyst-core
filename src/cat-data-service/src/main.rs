use clap::Parser;

pub mod cli;
pub mod service;
pub mod settings;

#[tokio::main]
async fn main() -> Result<(), cli::Error> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    cli::Cli::parse().exec().await?;
    Ok(())
}
