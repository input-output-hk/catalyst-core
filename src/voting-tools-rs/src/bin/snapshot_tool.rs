use std::{fs::File, io::BufWriter};

use clap::Parser;
use color_eyre::Result;
use mainnet_lib::InMemoryDbSync;
use tracing::info;
use voting_tools_rs::test_api::MockDbProvider;
use voting_tools_rs::{
    voting_power, Args, Db, DbConfig, DryRunCommand, SnapshotEntry, VotingPowerArgs,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let Args {
        db,
        db_user,
        db_host,
        db_pass,
        min_slot,
        max_slot,
        out_file,
        pretty,
        dry_run,
        network_id,
        expected_voting_purpose,
        ..
    } = Args::parse();

    let db_config = DbConfig {
        name: db,
        user: db_user,
        host: db_host,
        password: db_pass,
    };

    let mut args = VotingPowerArgs::default();
    args.min_slot = min_slot;
    args.max_slot = max_slot;
    args.network_id = network_id;
    args.expected_voting_purpose = expected_voting_purpose;

    let outputs = load(db_config, dry_run, args)?;

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
    args: VotingPowerArgs,
) -> Result<Vec<SnapshotEntry>> {
    let (result, _) = match dry_run {
        Some(DryRunCommand::DryRun { mock_json_file }) => {
            let db = MockDbProvider::from(InMemoryDbSync::restore(mock_json_file)?);
            voting_power(db, args)
        }
        None => {
            let db = Db::connect(real_db_config)?;
            voting_power(db, args)
        }
    }?;

    Ok(result)
}
