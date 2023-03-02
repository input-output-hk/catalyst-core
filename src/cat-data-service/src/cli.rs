use clap::Parser;

#[derive(thiserror::Error, Debug)]
pub enum Error {}

#[derive(Parser)]
pub enum Cli {
    Run,
}

impl Cli {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Self::Run => Ok(()),
        }
    }
}
