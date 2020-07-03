use structopt::StructOpt;

use vit_servicing_station_lib::{
    db, server, server::settings as server_settings, server::settings::ServiceSettings, v0,
};

#[tokio::main]
async fn main() {
    // load settings from command line (defaults to env variables)
    let mut settings: ServiceSettings = ServiceSettings::from_args();
    // dump settings and exit if specified
    if let Some(settings_file) = &settings.out_settings_file {
        server_settings::dump_settings_to_file(settings_file, &settings).unwrap_or_else(|e| {
            println!("Error writing settings to file {}: {}", settings_file, e);
            std::process::exit(1)
        });
        return;
    }

    // load settings from file if specified
    if let Some(settings_file) = &settings.in_settings_file {
        settings = server_settings::load_settings_from_file(settings_file).unwrap_or_else(|e| {
            println!("Error loading settings from file {}: {}", settings_file, e);
            std::process::exit(1)
        });
    };

    // load db pool
    let db_pool = db::load_db_connection_pool(&settings.db_url).unwrap_or_else(|e| {
        println!("Error connecting to database: {}", e);
        std::process::exit(1)
    });

    // load block0
    let block0 = std::fs::read(&settings.block0_path).unwrap_or_else(|e| {
        println!("Error loading block0 from {}: {}", &settings.block0_path, e,);
        std::process::exit(1)
    });

    let context = v0::context::new_shared_context(db_pool, block0);

    let app = v0::filter(context).await;

    println!(
        "Running server at {}, database located at {}",
        settings.address, settings.db_url
    );

    // run server with settings
    server::start_server(app, Some(settings)).await
}
