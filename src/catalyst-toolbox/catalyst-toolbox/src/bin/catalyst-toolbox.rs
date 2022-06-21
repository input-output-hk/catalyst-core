use structopt::StructOpt;

pub mod cli;

fn main() -> color_eyre::Result<()> {
    env_logger::try_init()?;
    color_eyre::install()?;
    cli::Cli::from_args().exec()?;
    Ok(())
}
