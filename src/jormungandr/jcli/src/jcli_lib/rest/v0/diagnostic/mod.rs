use crate::jcli_lib::rest::{Error, RestArgs};
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Diagnostic {
    /// Get system diagnostic information
    Get {
        #[clap(flatten)]
        args: RestArgs,
    },
}

impl Diagnostic {
    pub fn exec(self) -> Result<(), Error> {
        let args = match self {
            Diagnostic::Get { args } => args,
        };
        let response = args
            .client()?
            .get(&["api", "v0", "diagnostic"])
            .execute()?
            .text()?;
        println!("{}", response);
        Ok(())
    }
}
