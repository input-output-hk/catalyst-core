use std::path::{Path, PathBuf};

use hersir::controller::Context;

pub trait ContextExtension {
    fn empty_from_dir<P: AsRef<Path>>(dir: P) -> Context;
}

impl ContextExtension for Context {
    fn empty_from_dir<P: AsRef<Path>>(dir: P) -> Self {
        Context {
            jormungandr: PathBuf::new(),
            testing_directory: PathBuf::new().into(),
            generate_documentation: true,
            session_mode: todo!("choose session mode"),
        }
    }
}
