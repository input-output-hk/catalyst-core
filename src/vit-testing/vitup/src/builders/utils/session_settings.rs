use std::path::Path;

use hersir::config::SessionSettings;

pub trait SessionSettingsExtension {
    fn empty_from_dir<P: AsRef<Path>>(dir: P) -> SessionSettings;
}

impl SessionSettingsExtension for SessionSettings {
    #[allow(clippy::field_reassign_with_default)]
    fn empty_from_dir<P: AsRef<Path>>(dir: P) -> Self {
        let mut session_settings = Self::default();
        session_settings.root = Some(dir.as_ref().to_path_buf());
        session_settings
    }
}
