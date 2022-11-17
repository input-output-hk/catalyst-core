use std::{fs::File, io::BufWriter};

use clap::Parser;
use color_eyre::Result;
use mainnet_lib::InMemoryDbSync;
use tracing::debug;
use voting_tools_rs::test_api::MockDbProvider;
use voting_tools_rs::{voting_power, Args, DataProvider, Db, DbConfig, DryRunCommand};

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let Args {
        testnet_magic,
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

    let db = get_data_provider(db_config, dry_run)?;

    let outputs = voting_power(Box::leak(db), min_slot_no, max_slot_no, testnet_magic)?;

    debug!("calculated {} outputs", outputs.len());

    let file = File::options().write(true).create(true).open(out_file)?;
    let writer = BufWriter::new(file);

    match pretty {
        true => serde_json::to_writer_pretty(writer, &outputs),
        false => serde_json::to_writer(writer, &outputs),
    }?;

    Ok(())
}

fn get_data_provider(
    real_db_config: DbConfig,
    maybe_dry_run: Option<DryRunCommand>,
) -> Result<Box<dyn DataProvider>> {
    if let Some(dry_run) = maybe_dry_run {
        match dry_run {
            DryRunCommand::DryRun { mock_json_file } => Ok(Box::new(MockDbProvider::from(
                InMemoryDbSync::restore(&mock_json_file)?,
            ))),
        }
    } else {
        Ok(Box::new(Db::connect(real_db_config)?))
    }
}
