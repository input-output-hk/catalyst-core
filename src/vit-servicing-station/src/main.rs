#[macro_use]
extern crate diesel;
#[macro_use]
extern crate structopt;

pub mod db;
pub mod server;
pub mod server_settings;
pub mod utils;
pub mod v0;

use crate::server_settings::ServiceSettings;
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    // load settings from command line (defaults to env variables)
    let mut settings: ServiceSettings = ServiceSettings::from_args();
    // dump settings and exit if specified
    if let Some(settings_file) = &settings.out_settings_file {
        server_settings::dump_settings_to_file(settings_file, &settings)
            .unwrap_or_else(|e| panic!("Error writing settings to file {}: {}", settings_file, e));
        return;
    }

    // load settings from file if specified
    if let Some(settings_file) = &settings.in_settings_file {
        settings = server_settings::load_settings_from_file(settings_file).unwrap_or_else(|e| {
            panic!("Error loading settings from file {}: {}", settings_file, e)
        });
    };

    // run server with settings
    let db_pool =
        db::load_db_connection_pool(&settings.db_url).expect("Error connecting to database");
    let context = v0::context::new_shared_context(db_pool, &settings.block0_path);

    let app = v0::filter(context).await;

    println!(
        "Running server at {}, database located at {}",
        settings.address, settings.db_url
    );
    server::start_server(app, Some(settings)).await
}
