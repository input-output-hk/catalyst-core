mod deployment;
mod ideascale;
use crate::Result;
use deployment::DeploymentValidateCommand;
use ideascale::IdeascaleValidateCommand;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub enum ValidateCommand {
    Ideascale(IdeascaleValidateCommand),
    Deployment(DeploymentValidateCommand),
}

impl ValidateCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Ideascale(ideascale) => ideascale.exec(),
            Self::Deployment(deployment) => deployment.exec(),
        }
    }
}
