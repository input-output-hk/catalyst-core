use clap::Parser;
use color_eyre::Result;
use mainnet_lib::InMemoryDbSync;
use std::{fs::File, io::BufWriter};
use tracing::debug;
use voting_tools_rs::{test_api::MockDbProvider, voting_power, Args, DbConfig};

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
        ..
    } = Args::parse();

    let db_config = DbConfig {
        name: db,
        user: db_user,
        host: db_host,
        password: db_pass,
    };

    let db_sync_instance = InMemoryDbSync::restore(&*db_config.name)?;
    let db = MockDbProvider::from(db_sync_instance);
    let outputs = voting_power(&db, min_slot_no, max_slot_no, testnet_magic)?;

    debug!("calculated {} outputs", outputs.len());

    let file = File::options().write(true).create(true).open(out_file)?;
    let writer = BufWriter::new(file);

    match pretty {
        true => serde_json::to_writer_pretty(writer, &outputs),
        false => serde_json::to_writer(writer, &outputs),
    }?;

    Ok(())
}
