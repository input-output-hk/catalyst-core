use crate::utils::write_content;
use crate::Error;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct VoterRegistrationCli {
    path: PathBuf,
}

impl VoterRegistrationCli {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn generate_legacy_metadata<
        S: Into<String>,
        P: AsRef<Path>,
        R: AsRef<Path>,
        Q: AsRef<Path>,
    >(
        self,
        rewards_address: S,
        public_key_path: P,
        stake_skey_path: R,
        slot: u64,
        metadata_path: Q,
    ) -> Result<(), Error> {
        let mut command = Command::new(self.path);
        command
            .arg("--rewards-address")
            .arg(rewards_address.into())
            .arg("--vote-public-key")
            .arg(public_key_path.as_ref())
            .arg("--stake-signing-key")
            .arg(stake_skey_path.as_ref())
            .arg("--slot-no")
            .arg(slot.to_string())
            .arg("--json");

        println!("Running voter-registration: {:?}", command);

        let metadata_raw = command.output()?.stdout;
        let metadata_as_string = String::from_utf8(metadata_raw)?;
        write_content(&metadata_as_string, &metadata_path)?;
        Ok(())
    }

    pub fn generate_delegation_metadata<S: Into<String>, P: AsRef<Path>, R: AsRef<Path>>(
        self,
        rewards_address: S,
        delegations: HashMap<String, u32>,
        stake_skey_path: P,
        slot: u64,
        metadata_path: R,
    ) -> Result<(), Error> {
        let mut command = Command::new(self.path);

        command.arg("--rewards-address").arg(rewards_address.into());

        for (delegation, weight) in delegations {
            command
                .arg("--delegation")
                .arg(format!("{},{}", delegation, weight));
        }

        command
            .arg("--stake-signing-key")
            .arg(stake_skey_path.as_ref())
            .arg("--slot-no")
            .arg(slot.to_string())
            .arg("--json");

        println!("Running voter-registration: {:?}", command);

        let metadata_raw = command.output()?.stdout;
        let metadata_as_string = String::from_utf8(metadata_raw)?;
        write_content(&metadata_as_string, &metadata_path)?;
        Ok(())
    }
}
