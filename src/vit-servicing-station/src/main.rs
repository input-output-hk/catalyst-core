#[macro_use]
extern crate diesel;
#[macro_use]
extern crate structopt;

pub mod db;
pub mod server;
pub mod server_settings;
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
    let context = v0::context::new_default_context();
    let app = v0::filter(context);
    println!("Running server at {}", settings.address);
    server::start_server(app, Some(settings)).await
}
