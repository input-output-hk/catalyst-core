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
    let settings: ServiceSettings = ServiceSettings::from_args();
    let context = v0::context::new_default_context();
    let app = v0::filter(context);
    println!("Running server at {}", settings.address);
    server::start_server(app, Some(settings)).await
}
