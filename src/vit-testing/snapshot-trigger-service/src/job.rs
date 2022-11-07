use crate::config::{Configuration, JobParameters, NetworkType};
use crate::Error;
use scheduler_service_lib::JobRunner;
use std::path::PathBuf;
use std::process::Command;

pub struct SnapshotJobRunner(pub Configuration);

impl SnapshotJobRunner {
    fn print_with_password_hidden(&self, command: &Command) {
        let log_command = format!("{:?}", command);
        let pass = format!("--db-pass {}", &self.0.voting_tools.db_pass);
        println!(
            "Running command: {} ",
            log_command.replace(&pass, "--db-pass ***")
        );
    }

    pub fn crate_snapshot_output_file_name(&self, tag: &Option<String>) -> String {
        const SNAPSHOT_FILE: &str = "snapshot.json";

        if let Some(tag) = tag {
            format!("{}_{}", tag, SNAPSHOT_FILE)
        } else {
            SNAPSHOT_FILE.to_string()
        }
    }
}

impl JobRunner<JobParameters, (), crate::Error> for SnapshotJobRunner {
    fn start(&self, request: JobParameters, output_folder: PathBuf) -> Result<Option<()>, Error> {
        let mut command = self.0.voting_tools.command()?;
        match self.0.voting_tools.network {
            NetworkType::Mainnet => command.arg("--mainnet"),
            NetworkType::Testnet(magic) => command.arg("--testnet-magic").arg(magic.to_string()),
        };

        let output_filename = self.crate_snapshot_output_file_name(&request.tag);

        command
            .arg("--db")
            .arg(&self.0.voting_tools.db)
            .arg("--db-user")
            .arg(&self.0.voting_tools.db_user)
            .arg("--db-pass")
            .arg(&self.0.voting_tools.db_pass)
            .arg("--db-host")
            .arg(&self.0.voting_tools.db_host)
            .arg("--out-file")
            .arg(output_folder.join(output_filename))
            .arg("--scale")
            .arg(self.0.voting_tools.scale.to_string());

        if let Some(slot_no) = request.slot_no {
            command.arg("--slot-no").arg(slot_no.to_string());
        }

        self.print_with_password_hidden(&command);

        command.spawn()?.wait_with_output()?;
        Ok(Some(()))
    }
}
