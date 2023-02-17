use crate::jcli_lib::rest::{Error, RestArgs};
use clap::Parser;

/// Shutdown node
#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Shutdown {
    Post {
        #[clap(flatten)]
        args: RestArgs,
    },
}

impl Shutdown {
    pub fn exec(self) -> Result<(), Error> {
        let Shutdown::Post { args } = self;
        args.client()?.get(&["v0", "shutdown"]).execute()?;
        println!("Success");
        Ok(())
    }
}
