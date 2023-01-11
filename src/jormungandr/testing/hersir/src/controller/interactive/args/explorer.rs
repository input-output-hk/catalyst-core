use crate::controller::{Error, UserInteractionController};
use jormungandr_automation::jormungandr::explorer::configuration::ExplorerParams;
use jortestkit::prelude::InteractiveCommandError;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Explorer {
    /// Sends transaction
    Tip(ExplorerTip),
}

impl Explorer {
    pub fn exec(&self, controller: &mut UserInteractionController) -> Result<(), Error> {
        match self {
            Explorer::Tip(tip) => tip.exec(controller),
        }
    }
}

#[derive(Parser, Debug)]
pub struct ExplorerTip {
    #[clap(short = 'a', long = "alias")]
    pub alias: String,
}

impl ExplorerTip {
    pub fn exec(&self, controller: &mut UserInteractionController) -> Result<(), Error> {
        let node = controller
            .nodes()
            .iter()
            .find(|x| *x.alias() == self.alias)
            .ok_or_else(|| {
                InteractiveCommandError::UserError(format!("Node '{}' not found", self.alias))
            })?;
        println!(
            "{:#?}",
            node.explorer(ExplorerParams::default())?
                .client()
                .last_block()?
        );
        Ok(())
    }
}
