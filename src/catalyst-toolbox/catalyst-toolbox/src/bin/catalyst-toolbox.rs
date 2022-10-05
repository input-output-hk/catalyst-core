use structopt::StructOpt;

pub mod cli;

fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt().init();
    color_eyre::install()?;
    cli::Cli::from_args().exec()?;
    Ok(())
}
