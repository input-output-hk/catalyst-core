use microtype::secrecy::ExposeSecret;

use std::{collections::HashMap, path::Path};

use postgres::{Client, NoTls};
use std::{fs::File, io::BufWriter};

use clap::Parser;
use color_eyre::Result;
use tracing::{debug, info, Level};

use voting_tools_rs::{
    verify::{prefix_hex, Unregistered},
    voting_power, Args, DbConfig, DryRunCommand, InvalidRegistration, SnapshotEntry,
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
        enable_cip36_multiple_delegations,
        ..
    } = Args::parse();

    let db_config = DbConfig {
        name: db,
        user: db_user,
        host: db_host,
        password: db_pass,
        connect_timeout: 20,
        keepalives_idle: 900,
        keepalives_interval: 900,
        keepalives_retries: 8,
    };

    let mut args = VotingPowerArgs::default();
    args.min_slot = min_slot;
    args.max_slot = max_slot;
    args.network_id = network_id;
    args.expected_voting_purpose = expected_voting_purpose;
    args.cip_36_multidelegations = enable_cip36_multiple_delegations;

    let db_client_registrations = db_conn(db_config.clone())?;
    let db_client_stakes = db_conn(db_config)?;

    let (valids, invalids, unregistered) =
        load(dry_run, args, db_client_stakes, db_client_registrations)?;

    handle_invalids(&out_file, &invalids)?;

    handle_unregistered(&out_file, unregistered)?;

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
            "postgres://{0}{1}@{2}/{3}?connect_timeout={4}&keepalives=1&keepalives_idle={5}&keepalives_interval={6}&keepalives_retries={7}",
            db_config.user, password, db_config.host, db_config.name,db_config.connect_timeout,db_config.keepalives_idle,db_config.keepalives_interval,db_config.keepalives_retries
        ),
        NoTls,
    )
}

fn load(
    dry_run: Option<DryRunCommand>,
    args: VotingPowerArgs,
    db_client_stakes: Client,
    db_client_registrations: Client,
) -> Result<(Vec<SnapshotEntry>, Vec<InvalidRegistration>, Unregistered)> {
    if let Some(DryRunCommand::DryRun { mock_json_file }) = dry_run {
        info!("Using dryrun file: {}", mock_json_file.to_string_lossy());
        voting_power(db_client_stakes, db_client_registrations, args)
    } else {
        voting_power(db_client_stakes, db_client_registrations, args)
    }
}

/// Handle invalid registrations
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

/// Handle stake addresses that are not registered
fn handle_unregistered(path: &Path, unregistered: Unregistered) -> Result<()> {
    info!("handling unregistered");

    let unregistered = unregistered
        .into_iter()
        .map(|(key, value)| (prefix_hex(&key), value))
        .collect::<HashMap<String, u128>>();

    let path = path.with_extension("unregistered.json");

    tracing::warn!(
        "found unregistered stake addresses: writing to {}",
        path.to_string_lossy()
    );

    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &unregistered)?;

    Ok(())
}
