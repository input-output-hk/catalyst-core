use std::path::Path;

use clap::Parser;
use color_eyre::Result;
use db_sync_explorer::{connect, rest, Args, Config};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let Args { config, .. } = Args::parse();

    let configuration: Config = read_config(&config)?;
    let db_pool = connect(configuration.db)?;

    let address = configuration.address;
    let context = rest::v0::context::new_shared_context(db_pool, configuration.token);

    let app = rest::v0::filter(context).await;

    tracing::debug!("listening on {}", address);
    let (_, server) = warp::serve(app).bind_with_graceful_shutdown(
        address,
        vit_servicing_station_lib::server::signals::watch_signal_for_shutdown(),
    );
    server.await;
    Ok(())
}

pub fn read_config<P: AsRef<Path>>(config: P) -> Result<Config, color_eyre::Report> {
    let contents = std::fs::read_to_string(&config)?;
    serde_json::from_str(&contents).map_err(Into::into)
}
