use crate::jcli_lib::{
    rest::{Error, RestArgs},
    utils::OutputFormat,
};
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Plans {
    /// Get active vote plans list
    Get {
        #[clap(flatten)]
        args: RestArgs,
        #[clap(flatten)]
        output_format: OutputFormat,
    },
}

impl Plans {
    pub fn exec(self) -> Result<(), Error> {
        let Plans::Get {
            args,
            output_format,
        } = self;
        let response = args
            .client()?
            .get(&["api", "v0", "vote", "active", "plans"])
            .execute()?
            .json()?;
        let formatted = output_format.format_json(response)?;
        println!("{}", formatted);
        Ok(())
    }
}
