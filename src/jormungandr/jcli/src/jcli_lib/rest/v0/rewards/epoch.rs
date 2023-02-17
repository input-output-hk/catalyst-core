use crate::jcli_lib::rest::{Error, RestArgs};
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Epoch {
    /// Get rewards for epoch
    Get {
        #[clap(flatten)]
        args: RestArgs,
        /// Epoch number
        epoch: u32,
    },
}

impl Epoch {
    pub fn exec(self) -> Result<(), Error> {
        let Epoch::Get { args, epoch } = self;
        let response = args
            .client()?
            .get(&["v0", "rewards", "epoch", &epoch.to_string()])
            .execute()?
            .text()?;
        println!("{}", response);
        Ok(())
    }
}
