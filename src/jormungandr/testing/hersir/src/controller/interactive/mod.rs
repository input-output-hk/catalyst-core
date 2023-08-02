pub mod args;
mod command;
mod controller;

use clap::Parser;
pub use command::InteractiveCommand;
pub use controller::{do_for_all_alias, UserInteractionController};
pub use jortestkit::prelude::{ConsoleWriter, InteractiveCommandError, InteractiveCommandExec};
use std::ffi::OsStr;
pub struct JormungandrInteractiveCommandExec {
    pub controller: UserInteractionController,
}

impl InteractiveCommandExec for JormungandrInteractiveCommandExec {
    fn parse_and_exec(
        &mut self,
        tokens: Vec<String>,
        console: ConsoleWriter,
    ) -> std::result::Result<(), InteractiveCommandError> {
        let interactive = InteractiveCommand::parse_from(tokens.iter().map(OsStr::new));
        if let Err(err) = {
            match interactive {
                InteractiveCommand::Show(show) => {
                    show.exec(&mut self.controller);
                    Ok(())
                }
                InteractiveCommand::Spawn(spawn) => spawn.exec(&mut self.controller),
                InteractiveCommand::Exit => Ok(()),
                InteractiveCommand::Describe(describe) => describe.exec(&mut self.controller),
                InteractiveCommand::Send(send) => send.exec(&mut self.controller),
                InteractiveCommand::Explorer(explorer) => explorer.exec(&mut self.controller),
            }
        } {
            console.format_error(InteractiveCommandError::UserError(err.to_string()));
        }
        Ok(())
    }
}
