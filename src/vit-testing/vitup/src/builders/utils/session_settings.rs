use hersir::config::SessionSettings;
use std::path::Path;

pub trait SessionSettingsExtension {
    fn empty_from_dir<P: AsRef<Path>>(dir: P) -> SessionSettings;
}

impl SessionSettingsExtension for SessionSettings {
    #[allow(clippy::field_reassign_with_default)]
    fn empty_from_dir<P: AsRef<Path>>(dir: P) -> Self {
        let mut session_settings = Self::default();
        session_settings.generate_documentation = true;
        session_settings.root = dir.as_ref().to_path_buf().into();
        session_settings
    }
}
