use std::path::Path;
use std::str::FromStr;

use clap::Parser;
use color_eyre::Result;
use db_sync_explorer::{connect, rest, Args, Config, Db, MockProvider, Provider};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let Args {
        config,
        log_level,
        token,
        ..
    } = Args::parse();

    let subscriber = FmtSubscriber::builder()
        .with_file(false)
        .with_target(false)
        .with_max_level(LevelFilter::from_str(&log_level.to_string()).expect("invalid log level"))
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let mut configuration: Config = read_config(&config)?;
    let address = configuration.address;

    if token.is_some() {
        configuration.token = token;
    }

    let context = match &configuration.db {
        Db::Db(config) => {
            let db_pool = connect(config.clone())?;
            rest::v0::context::new_shared_real_context(Provider(db_pool), configuration)
        }
        Db::Mock(_config) => {
            rest::v0::context::new_shared_mocked_context(MockProvider::default(), configuration)
        }
    };

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
