mod build;
mod registration;

use std::process::Command;
pub struct StakeAddress {
    command: Command,
}

impl StakeAddress {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn build(mut self) -> build::StakeAddressBuilder {
        self.command.arg("build");
        build::StakeAddressBuilder::new(self.command)
    }

    pub fn register_certificate(mut self) -> registration::RegisterCertificate {
        self.command.arg("register-certificate");
        registration::RegisterCertificate::new(self.command)
    }
}
