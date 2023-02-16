use crate::jcli_lib::{
    rest::{Error, RestArgs},
    utils::OutputFormat,
};
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Utxo {
    /// hex-encoded ID of the transaction fragment
    fragment_id: String,

    /// index of the transaction output
    output_index: u8,

    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
enum Subcommand {
    /// Get UTxO details
    Get {
        #[clap(flatten)]
        output_format: OutputFormat,

        #[clap(flatten)]
        args: RestArgs,
    },
}

impl Utxo {
    pub fn exec(self) -> Result<(), Error> {
        let Subcommand::Get {
            args,
            output_format,
        } = self.subcommand;
        let response = args
            .client()?
            .get(&[
                "api",
                "v0",
                "utxo",
                &self.fragment_id,
                &self.output_index.to_string(),
            ])
            .execute()?
            .json()?;
        let formatted = output_format.format_json(response)?;
        println!("{}", formatted);
        Ok(())
    }
}
