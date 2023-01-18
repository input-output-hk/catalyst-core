use std::{fs::File, io::BufWriter};

use clap::Parser;
use color_eyre::Result;
use mainnet_lib::InMemoryDbSync;
use tracing::info;
use voting_tools_rs::test_api::MockDbProvider;
use voting_tools_rs::{
    voting_power, Args, Db, DbConfig, DryRunCommand, SlotNo, SnapshotEntry,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let Args {
        db,
        db_user,
        db_host,
        db_pass,
        min_slot_no,
        max_slot_no,
        out_file,
        pretty,
        dry_run,
        ..
    } = Args::parse();

    let db_config = DbConfig {
        name: db,
        user: db_user,
        host: db_host,
        password: db_pass,
    };

    let outputs = load(db_config, dry_run, min_slot_no, max_slot_no)?;

    info!("calculated {} outputs", outputs.len());

    let file = File::options().write(true).create(true).open(out_file)?;
    let writer = BufWriter::new(file);

    match pretty {
        true => serde_json::to_writer_pretty(writer, &outputs),
        false => serde_json::to_writer(writer, &outputs),
    }?;

    Ok(())
}

fn load(
    real_db_config: DbConfig,
    dry_run: Option<DryRunCommand>,
    min_slot: Option<SlotNo>,
    max_slot: Option<SlotNo>,
) -> Result<Vec<SnapshotEntry>> {
    match dry_run {
        Some(DryRunCommand::DryRun { mock_json_file }) => {
            let db = MockDbProvider::from(InMemoryDbSync::restore(mock_json_file)?);
            voting_power(db, min_slot, max_slot)
        }
        None => {
            let db = Db::connect(real_db_config)?;
            voting_power(db, min_slot, max_slot)
        }
    }
}
