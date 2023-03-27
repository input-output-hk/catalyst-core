use microtype::secrecy::ExposeSecret;

use std::path::Path;

use postgres::{Client, NoTls};
use std::{fs::File, io::BufWriter};

use clap::Parser;
use color_eyre::Result;
use mainnet_lib::InMemoryDbSync;
use tracing::{debug, info, Level};
use voting_tools_rs::test_api::MockDbProvider;

use voting_tools_rs::{
    voting_power_beta, Args, Db, DbConfig, DryRunCommand, InvalidRegistration, SnapshotEntry,
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
        .with_max_level(Level::INFO /*DEBUG*/)
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

    let client = db_conn(db_config.clone())?;

    let (valids, invalids) = load(db_config, dry_run, args, client)?;

    handle_invalids(&out_file, &invalids)?;

    info!(
        "calculated {} valids invalids {}",
        valids.len(),
        invalids.len()
    );

    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_file)?;
    let writer = BufWriter::new(file);

    // Snapshots are so large that non-pretty output is effectively unusable.
    // So ONLY do pretty formatted output.
    serde_json::to_writer_pretty(writer, &valids)?;

    Ok(())
}

fn db_conn(db_config: DbConfig) -> Result<Client, postgres::Error> {
    let password = db_config
        .password
        .map(|p| format!(":{}", p.expose_secret()))
        .unwrap_or_default();

    Client::connect(
        &format!(
            "postgres://{0}{1}@{2}/{3}",
            db_config.user, password, db_config.host, db_config.name,
        ),
        NoTls,
    )
}

fn load(
    real_db_config: DbConfig,
    dry_run: Option<DryRunCommand>,
    args: VotingPowerArgs,
    registration_client: Client,
) -> Result<(Vec<SnapshotEntry>, Vec<InvalidRegistration>)> {
    if let Some(DryRunCommand::DryRun { mock_json_file }) = dry_run {
        info!("Using dryrun file: {}", mock_json_file.to_string_lossy());
        let db = MockDbProvider::from(InMemoryDbSync::restore(mock_json_file)?);
        voting_power_beta(db, registration_client, args)
    } else {
        let db_loc = &real_db_config.host;
        let db_user = &real_db_config.name;
        info!(
            "Connecting to Postgresql at {}:xxxxxxxx@{}.",
            db_user, db_loc
        );
        let db = Db::connect(real_db_config)?;
        voting_power_beta(db, registration_client, args)
    }
}

fn handle_invalids(path: &Path, invalids: &[InvalidRegistration]) -> Result<()> {
    info!("handling invalids");
    if invalids.is_empty() {
        return Ok(());
    }

    let path = path.with_extension("errors.json");

    tracing::warn!(
        "found invalid registrations: writing to {}",
        path.to_string_lossy()
    );

    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, invalids)?;

    Ok(())
}
