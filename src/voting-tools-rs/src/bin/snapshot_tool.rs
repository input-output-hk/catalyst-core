use std::{fs::File, io::BufWriter};

use clap::Parser;
use color_eyre::Result;
use voting_tools_rs::{voting_power, Args, Db, DbConfig};

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

    let db = Db::connect(db_config)?;
    let outputs = voting_power(&db, min_slot_no, max_slot_no, testnet_magic)?;

    let file = File::options().write(true).create(true).open(out_file)?;
    let writer = BufWriter::new(file);

    println!("{outputs:#?}");
    println!("{}", outputs.len());

    match pretty {
        true => serde_json::to_writer_pretty(writer, &outputs),
        false => serde_json::to_writer(writer, &outputs),
    }?;

    Ok(())
}
