use crate::jcli_lib::{
    rest::{Error, RestArgs},
    utils::OutputFormat,
};
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum StakePools {
    /// Get stake pool IDs
    Get {
        #[clap(flatten)]
        args: RestArgs,
        #[clap(flatten)]
        output_format: OutputFormat,
    },
}

impl StakePools {
    pub fn exec(self) -> Result<(), Error> {
        let StakePools::Get {
            args,
            output_format,
        } = self;
        let response = args
            .client()?
            .get(&["v0", "stake_pools"])
            .execute()?
            .json()?;
        let formatted = output_format.format_json(response)?;
        println!("{}", formatted);
        Ok(())
    }
}
