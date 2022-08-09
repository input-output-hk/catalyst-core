use clap::Parser;
use color_eyre::Result;
use voting_tools_rs::cli::Args;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let _args = Args::parse();

    Ok(())
}
