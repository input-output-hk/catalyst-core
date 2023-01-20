use super::UserInteractionController;
use crate::{controller::Error, style};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct SendTransaction {
    #[clap(short = 'f', long = "from")]
    pub from: String,
    #[clap(short = 't', long = "to")]
    pub to: String,
    #[clap(short = 'v', long = "via")]
    pub via: String,
    #[clap(short = 'a', long = "ada")]
    pub ada: Option<u64>,
}

impl SendTransaction {
    pub fn exec(&self, controller: &mut UserInteractionController) -> Result<(), Error> {
        let mem_pool_check = controller.send_transaction(
            &self.from,
            &self.to,
            &self.via,
            self.ada.unwrap_or(100).into(),
        )?;
        println!(
            "{}",
            style::info.apply_to(format!(
                "fragment '{}' successfully sent",
                mem_pool_check.fragment_id()
            ))
        );
        Ok(())
    }
}
