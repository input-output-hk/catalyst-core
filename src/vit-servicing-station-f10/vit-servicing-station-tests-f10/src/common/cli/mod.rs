use super::startup::get_cli_exe;
use assert_cmd::assert::OutputAssertExt;
mod commands;
pub use commands::VitCliCommand;
use jortestkit::process::output_extensions::ProcessOutput;
use std::path::PathBuf;
pub struct VitCli {
    exe: PathBuf,
}

impl Default for VitCli {
    fn default() -> Self {
        Self::new(get_cli_exe())
    }
}

impl VitCli {
    pub fn new(exe: PathBuf) -> Self {
        Self { exe }
    }

    pub fn generate_tokens(&self, n: u32) -> Vec<String> {
        let vit_command: VitCliCommand = VitCliCommand::new(self.exe.clone());
        vit_command
            .api_token()
            .generate()
            .n(n)
            .build()
            .assert()
            .get_output()
            .as_multi_line()
    }
}
