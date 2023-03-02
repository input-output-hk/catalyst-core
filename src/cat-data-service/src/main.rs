use clap::Parser;

mod cli;
mod v0;

#[tokio::main]
async fn main() -> Result<(), cli::Error> {
    cli::Cli::parse().exec()?;
    Ok(())
}
