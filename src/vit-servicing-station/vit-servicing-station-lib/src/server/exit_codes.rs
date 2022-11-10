use std::process::{ExitCode, Termination};

pub enum ApplicationExitCode {
    Success = 0,
    WriteSettingsError = 10,
    LoadSettingsError,
    DbConnectionError,
    ServiceVersionError,
    SnapshotWatcherError,
    EmptyBlock0FolderError,
}

impl Termination for ApplicationExitCode {
    fn report(self) -> ExitCode {
        ExitCode::from(self as u8)
    }
}
