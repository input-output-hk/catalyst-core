use std::path::Path;
use std::process::Command;
use tracing::debug;

pub struct RegisterCertificate {
    command: Command,
}

impl RegisterCertificate {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn stake_verification_key_file<P: AsRef<Path>>(
        mut self,
        stake_verification_key: P,
    ) -> Self {
        self.command
            .arg("--stake-verification-key-file")
            .arg(stake_verification_key.as_ref());
        self
    }

    pub fn out_file<P: AsRef<Path>>(mut self, out_file: P) -> Self {
        self.command.arg("--out-file").arg(out_file.as_ref());
        self
    }

    pub fn build(self) -> Command {
        debug!(
            "Cardano Cli - stake address registration certificate: {:?}",
            self.command
        );
        self.command
    }
}
