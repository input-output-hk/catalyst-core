use std::{ffi::OsStr, process::Command};

use color_eyre::eyre::Result;

pub(crate) fn exec_python_script(
    path: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<()> {
    Command::new("python").arg(path).args(args).output()?;
    Ok(())
}
