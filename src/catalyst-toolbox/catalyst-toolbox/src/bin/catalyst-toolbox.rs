use clap::Parser;

pub mod cli;

fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt().init();
    color_eyre::install()?;
    cli::Cli::parse().exec()?;
    Ok(())
}
