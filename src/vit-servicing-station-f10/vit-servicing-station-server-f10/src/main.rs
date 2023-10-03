use std::path::{Path, PathBuf};
use structopt::StructOpt;

use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;
use vit_servicing_station_lib_f10::{
    db, server, server::exit_codes::ApplicationExitCode, server::settings as server_settings,
    server::settings::ServiceSettings, v0,
};

fn check_and_build_proper_path(path: &Path) -> std::io::Result<()> {
    use std::fs;
    // create parent dirs if not exists
    fs::create_dir_all(path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Cannot create path tree {}", path.to_str().unwrap()),
        )
    })?)?;
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path)?;
    Ok(())
}

fn config_tracing(
    level: server_settings::LogLevel,
    pathbuf: Option<PathBuf>,
) -> Result<WorkerGuard, std::io::Error> {
    if let Some(path) = pathbuf {
        // check path integrity
        // we try opening the file since tracing appender would just panic instead of
        // returning an error
        check_and_build_proper_path(&path)?;

        let file_appender = tracing_appender::rolling::never(
            path.parent().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Log file path `{}` is invalid.", path.display()),
                )
            })?,
            path.file_name().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Log file path `{}` doesn't contain a valid file name.",
                        path.display()
                    ),
                )
            })?,
        );

        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_max_level(level)
            .init();
        Ok(guard)
    } else {
        let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
        tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_max_level(level)
            .init();
        Ok(guard)
    }
}

#[tokio::main]
async fn main() {
    // load settings from command line (defaults to env variables)
    let mut settings: ServiceSettings = ServiceSettings::from_args();

    // load settings from file if specified
    if let Some(settings_file) = &settings.in_settings_file {
        let in_file_settings = server_settings::load_settings_from_file(settings_file)
            .unwrap_or_else(|e| {
                error!("Error loading settings from file {}, {}", settings_file, e);
                std::process::exit(ApplicationExitCode::LoadSettingsError.into())
            });
        // merge input file settings override by cli arguments
        settings = in_file_settings.override_from(&settings);
    }

    // dump settings and exit if specified
    if let Some(settings_file) = &settings.out_settings_file {
        server_settings::dump_settings_to_file(settings_file, &settings).unwrap_or_else(|e| {
            error!("Error writing settings to file {}: {}", settings_file, e);
            std::process::exit(ApplicationExitCode::WriteSettingsError.into())
        });
        std::process::exit(0);
    }

    // setup logging
    let _guard = config_tracing(
        settings.log.log_level.unwrap_or_default(),
        settings.log.log_output_path.clone(),
    )
    .unwrap_or_else(|e| {
        error!("Error setting up logging: {}", e);
        std::process::exit(ApplicationExitCode::LoadSettingsError.into())
    });

    // Check db file exists (should be here only for current sqlite db backend)
    if !std::path::Path::new(&settings.db_url).exists() {
        error!("DB file {} not found.", &settings.db_url);
        std::process::exit(ApplicationExitCode::DbConnectionError.into())
    }
    // load db pool
    let db_pool = db::load_db_connection_pool(&settings.db_url).unwrap_or_else(|e| {
        error!("Error connecting to database: {}", e);
        std::process::exit(ApplicationExitCode::DbConnectionError.into())
    });

    let context =
        v0::context::new_shared_context(db_pool, &settings.block0_path, &settings.service_version);

    let app = v0::filter(context, settings.enable_api_tokens).await;

    info!(
        "Running server at {}, database located at {}",
        settings.address, settings.db_url
    );

    // run server with settings
    server::start_server(app, Some(settings)).await
}
