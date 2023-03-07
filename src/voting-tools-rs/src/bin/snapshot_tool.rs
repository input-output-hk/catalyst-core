use std::path::Path;
use std::{fs::File, io::BufWriter};

use clap::Parser;
use color_eyre::Result;
use mainnet_lib::InMemoryDbSync;
use tracing::{debug, info, Level};
use voting_tools_rs::test_api::MockDbProvider;
use voting_tools_rs::{
    voting_power, Args, Db, DbConfig, DryRunCommand, InvalidRegistration, SnapshotEntry,
    VotingPowerArgs,
};

fn main() -> Result<()> {
    color_eyre::install()?;

    // Configure a custom event formatter
    let format = tracing_subscriber::fmt::format()
        .with_level(true) // don't include levels in formatted output
        .with_target(true) // don't include targets
        .with_thread_ids(true) // include the thread ID of the current thread
        .with_thread_names(true) // include the name of the current thread
        .compact(); // use the `Compact` formatting style.

    // Create a `fmt` subscriber that uses our custom event format, and set it
    // as the default.
    tracing_subscriber::fmt()
        .event_format(format)
        .with_max_level(Level::DEBUG)
        .init();

    info!("Snapshot Tool.");
    debug!("Debug Logs Enabled!");

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

    let (outputs, invalids) = load(db_config, dry_run, args)?;

    info!("calculated {} outputs", outputs.len());

    handle_invalids(&out_file, &invalids)?;

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
) -> Result<(Vec<SnapshotEntry>, Vec<InvalidRegistration>)> {
    let result = match dry_run {
        Some(DryRunCommand::DryRun { mock_json_file }) => {
            info!("using dryrun file: {}", mock_json_file.to_string_lossy());
            let db = MockDbProvider::from(InMemoryDbSync::restore(mock_json_file)?);
            voting_power(db, args)
        }
        None => {
            info!("using real db");
            let db = Db::connect(real_db_config)?;
            voting_power(db, args)
        }
    }?;

    Ok(result)
}

fn handle_invalids(path: &Path, invalids: &[InvalidRegistration]) -> Result<()> {
    info!("handling invalids");
    if invalids.is_empty() {
        return Ok(());
    }

    let path = path.with_file_name("voting_tool_error");

    tracing::warn!(
        "found invalid registrations: writing to {}",
        path.to_string_lossy()
    );

    let file = File::options().write(true).create(true).open(path)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, invalids)?;

    Ok(())
}
