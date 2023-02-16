use crate::jcli_lib::rest::{Error, RestArgs};
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Tip {
    /// Get tip ID
    Get {
        #[clap(flatten)]
        args: RestArgs,
    },
}

impl Tip {
    pub fn exec(self) -> Result<(), Error> {
        let args = match self {
            Tip::Get { args } => args,
        };
        let response = args
            .client()?
            .get(&["api", "v0", "tip"])
            .execute()?
            .text()?;
        println!("{}", response);
        Ok(())
    }
}
