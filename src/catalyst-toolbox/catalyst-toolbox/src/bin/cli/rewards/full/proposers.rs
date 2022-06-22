use std::{ffi::OsStr, path::Path};

use color_eyre::Result;

use super::python::exec_python_script;

pub(super) fn proposers_rewards(
    proposer_reward_script: &Path,
    block0: &Path,
    output: &Path,
    stake_threshold: f64,
    approval_threshold: f64,
    proposals: &Path,
    active_voteplans: &Path,
    challenges: &Path,
) -> Result<()> {
    exec_python_script(
        proposer_reward_script,
        [
            OsStr::new("--block0-path"),
            block0.as_ref(),
            OsStr::new("--output-file"),
            output.as_ref(),
            OsStr::new("--total-stake-threshold"),
            OsStr::new(&format!("{stake_threshold}")),
            OsStr::new("--approval-threshold"),
            OsStr::new(&format!("{approval_threshold}")),
            OsStr::new("--proposals-path"),
            proposals.as_ref(),
            OsStr::new("--active-voteplans-path"),
            active_voteplans.as_ref(),
            OsStr::new("--challenges-path"),
            challenges.as_ref(),
        ],
    )
}
