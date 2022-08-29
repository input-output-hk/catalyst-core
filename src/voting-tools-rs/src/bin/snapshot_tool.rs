use std::{fs::File, io::BufWriter};

use clap::Parser;
use color_eyre::Result;
use voting_tools_rs::{run, Args, DbConfig};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let Args {
        testnet_magic,
        db,
        db_user,
        db_host,
        db_pass,
        slot_no,
        out_file,
        ..
    } = Args::parse();
    let db_config = DbConfig {
        name: db,
        user: db_user,
        host: db_host,
        password: db_pass,
    };
    let results = run(db_config, slot_no, testnet_magic).await?;

    let file = File::options().write(true).open(out_file)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, &results)?;

    Ok(())
}
