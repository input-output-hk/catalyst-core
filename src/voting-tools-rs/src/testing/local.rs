use std::{fs::read_to_string, process::Command};

use crate::{
    model::{Output, SlotNo},
    run, DbConfig,
};
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use tempdir::TempDir;

pub fn get_db_config() -> Result<DbConfig> {
    let root = env!("CARGO_MANIFEST_DIR");
    let file = format!("{root}/test_db.json");
    let json = std::fs::read_to_string(file)?;
    Ok(serde_json::from_str(&json)?)
}

pub fn get_haskell(
    DbConfig {
        name, user, host, ..
    }: &DbConfig,
    slot_no: Option<SlotNo>,
) -> Result<Vec<Output>> {
    let haskell_tool = std::env::var("HASKELL_TOOL")
        .context("$HASKELL_TOOL should be set to a path to the haskell tool")?;
    let dir = TempDir::new("voting")?;
    let output = dir.path().join("output");

    let mut args = vec![
        "--mainnet".to_string(),
        "--db".to_string(),
        name.to_string(),
        "--db-user".to_string(),
        user.to_string(),
        "--db-host".to_string(),
        host.to_string(),
        "--out-file".to_string(),
        output.to_str().ok_or(eyre!(""))?.into(),
    ];

    if let Some(slot_no) = slot_no {
        args.extend(["--slot-no".into(), slot_no.to_string()])
    }

    Command::new(haskell_tool).args(args).status()?;

    let output_str = read_to_string(output)?;
    let result = serde_json::from_str(&output_str)?;

    Ok(result)
}

pub fn get_rust(db_config: &DbConfig, slot_no: Option<SlotNo>) -> Result<Vec<Output>> {
    todo!()
}
