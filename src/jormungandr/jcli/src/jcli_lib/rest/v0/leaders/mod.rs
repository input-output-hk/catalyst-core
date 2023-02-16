use crate::jcli_lib::{
    rest::{Error, RestArgs},
    utils::OutputFormat,
};
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Leaders {
    /// Leadership log operations
    #[clap(subcommand)]
    Logs(GetLogs),
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum GetLogs {
    /// Get leadership log
    Get {
        #[clap(flatten)]
        args: RestArgs,
        #[clap(flatten)]
        output_format: OutputFormat,
    },
}

impl Leaders {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Leaders::Logs(GetLogs::Get {
                args,
                output_format,
            }) => get_logs(args, output_format),
        }
    }
}

fn get_logs(args: RestArgs, output_format: OutputFormat) -> Result<(), Error> {
    let response = args
        .client()?
        .get(&["api", "v0", "leaders", "logs"])
        .execute()?
        .json()?;
    let formatted = output_format.format_json(response)?;
    println!("{}", formatted);
    Ok(())
}
