use jormungandr_scenario_tests::ProgressBarMode;
use jormungandr_scenario_tests::{Context, Seed};
use std::path::{Path, PathBuf};

pub trait ContextExtension {
    fn empty_from_dir<P: AsRef<Path>>(dir: P) -> Context;
}

impl ContextExtension for Context {
    fn empty_from_dir<P: AsRef<Path>>(dir: P) -> Self {
        Context::new(
            Seed::generate(rand::rngs::OsRng),
            PathBuf::new(),
            PathBuf::new(),
            Some(dir.as_ref().to_path_buf()),
            true,
            ProgressBarMode::None,
            "info".to_string(),
        )
    }
}
