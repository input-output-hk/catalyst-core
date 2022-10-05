mod build;
mod registration;

use std::process::Command;
pub struct StakeAddressCommand {
    command: Command,
}

impl StakeAddressCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn build(mut self) -> build::StakeAddressBuildCommand {
        self.command.arg("build");
        build::StakeAddressBuildCommand::new(self.command)
    }

    pub fn register_certificate(mut self) -> registration::RegisterCertificateCommand {
        self.command.arg("register-certificate");
        registration::RegisterCertificateCommand::new(self.command)
    }
}
