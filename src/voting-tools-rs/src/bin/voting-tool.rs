use clap::Parser;
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    voting_tools_rs::cli::Args::parse();

    Ok(())
}
