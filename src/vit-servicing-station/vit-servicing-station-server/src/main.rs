use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::Parser;
use opentelemetry_otlp::WithExportConfig;
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::LevelFilter;
use vit_servicing_station_lib::server::exit_codes::ApplicationExitCode;
use vit_servicing_station_lib::{
    db,
    server::{
        self,
        settings::{self as server_settings, ServiceSettings},
    },
    v0,
};

fn check_and_build_proper_path(path: &Path) -> std::io::Result<()> {
    // create parent dirs if not exists
    fs::create_dir_all(path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Cannot create path tree {}", path.to_str().unwrap()),
        )
    })?)?;
    fs::OpenOptions::new().create(true).write(true).truncate(true).open(path)?;
    Ok(())
}

struct LogGuard {
    _nonblocking_worker_guard: WorkerGuard,
}

impl Drop for LogGuard {
    fn drop(&mut self) {
        tracing::trace!("Shutting down opentelemetry trace provider");
        opentelemetry::global::shutdown_tracer_provider();
    }
}

#[derive(Debug, thiserror::Error)]
enum ConfigTracingError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to initialize tracing subscriber: {0}")]
    InitSubscriber(#[from] tracing_subscriber::util::TryInitError),
    #[error("failed to install opentelemetry pipeline")]
    InstallOpenTelemetryPipeLine(#[from] opentelemetry::trace::TraceError),
}

fn config_tracing(
    level: server_settings::LogLevel,
    pathbuf: Option<PathBuf>,
    trace_collector_endpoint: Option<url::Url>,
) -> Result<LogGuard, ConfigTracingError> {
    use tracing_subscriber::prelude::*;

    let otel_layer = if let Some(endpoint) = trace_collector_endpoint {
        let otel_tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint),
            )
            .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
                opentelemetry::sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    "jormungandr",
                )]),
            ))
            .install_batch(opentelemetry::runtime::Tokio)?;

        Some(tracing_opentelemetry::layer().with_tracer(otel_tracer))
    } else {
        None
    };

    let subscriber = tracing_subscriber::registry()
        .with(LevelFilter::from(level))
        .with(otel_layer);

    let guard = if let Some(path) = pathbuf {
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

        let layer = tracing_subscriber::fmt::layer().with_writer(non_blocking);

        subscriber.with(layer).try_init()?;

        guard
    } else {
        let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());

        let layer = tracing_subscriber::fmt::layer().with_writer(non_blocking);

        subscriber.with(layer).try_init()?;

        guard
    };

    Ok(LogGuard {
        _nonblocking_worker_guard: guard,
    })
}

#[tokio::main]
async fn main() -> ApplicationExitCode {
    // load settings from command line (defaults to env variables)
    let mut settings: ServiceSettings = ServiceSettings::parse();

    // load settings from file if specified
    if let Some(settings_file) = &settings.in_settings_file {
        let in_file_settings = match server_settings::load_settings_from_file(settings_file) {
            Ok(s) => s,
            Err(e) => {
                error!("Error loading settings from file {}, {}", settings_file, e);
                return ApplicationExitCode::LoadSettingsError;
            }
        };

        // merge input file settings override by cli arguments
        settings = in_file_settings.override_from(&settings);
    }

    // dump settings and exit if specified
    if let Some(settings_file) = &settings.out_settings_file {
        if let Err(e) = server_settings::dump_settings_to_file(settings_file, &settings) {
            error!("Error writing settings to file {}: {}", settings_file, e);
            return ApplicationExitCode::WriteSettingsError;
        }

        return ApplicationExitCode::Success;
    }

    // setup logging
    let _guard = match config_tracing(
        settings.log.log_level.unwrap_or_default(),
        settings.log.log_output_path.clone(),
        settings.log.trace_collector_endpoint.clone(),
    ) {
        Ok(g) => g,
        Err(e) => {
            error!("Error setting up logging: {}", e);
            return ApplicationExitCode::LoadSettingsError;
        }
    };

    // load db pool
    let db_pool = match db::load_db_connection_pool(&settings.db_url) {
        Ok(d) => d,
        Err(e) => {
            error!("Error connecting to database: {}", e);
            return ApplicationExitCode::DbConnectionError;
        }
    };

    let paths: Vec<PathBuf> = if let Some(single_block0_path) = &settings.block0_path {
        vec![PathBuf::from_str(single_block0_path).unwrap()]
    } else {
        let block0_paths = match settings.block0_paths.as_ref() {
            Some(p) => p,
            None => return ApplicationExitCode::EmptyBlock0FolderError,
        };

        fs::read_dir(block0_paths)
            .unwrap()
            .filter_map(|path| {
                let path = path.unwrap().path();
                match path.extension() {
                    Some(p) if p == "bin" => Some(path.to_path_buf()),
                    _ => None,
                }
            })
            .collect()
    };

    if paths.is_empty() {
        return ApplicationExitCode::EmptyBlock0FolderError;
    }

    let context = v0::context::new_shared_context(db_pool, paths, &settings.service_version);

    let app = v0::filter(context, settings.enable_api_tokens).await;

    info!(
        "Running server at {}, database located at {}",
        settings.address, settings.db_url
    );

    // run server with settings
    server::start_server(app, Some(settings)).await;
    ApplicationExitCode::Success
}
