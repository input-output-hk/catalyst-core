use opentelemetry_otlp::WithExportConfig;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    fs, io,
    path::PathBuf,
    str::FromStr,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// Format of the logger.
pub enum LogFormat {
    Plain,
    Json,
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Plain
    }
}

impl Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogFormat::Plain => "plain",
            LogFormat::Json => "json",
        };
        f.write_str(s)
    }
}

impl FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "plain" => Ok(LogFormat::Plain),
            "json" => Ok(LogFormat::Json),
            other => Err(format!("unknown log format '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// Output of the logger.
pub enum LogOutput {
    Stdout,
    Stderr,
    File(PathBuf),
}

impl FromStr for LogOutput {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_prefix('@') {
            return Ok(Self::File(PathBuf::from(stripped)));
        }

        match s.trim().to_lowercase().as_str() {
            "stdout" => Ok(LogOutput::Stdout),
            "stderr" => Ok(LogOutput::Stderr),
            other => Err(format!("unknown log output '{}'", other)),
        }
    }
}

impl Default for LogOutput {
    fn default() -> Self {
        Self::Stderr
    }
}

pub struct LogGuard {
    _nonblocking_worker_guard: tracing_appender::non_blocking::WorkerGuard,
}

impl Drop for LogGuard {
    fn drop(&mut self) {
        tracing::trace!("shutting down opentelemetry trace provider");
        opentelemetry::global::shutdown_tracer_provider();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogSettings {
    pub level: LevelFilter,
    pub format: LogFormat,
    pub output: LogOutput,
    pub trace_collector_endpoint: Option<Url>,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            level: LevelFilter::TRACE,
            format: Default::default(),
            output: Default::default(),
            trace_collector_endpoint: None,
        }
    }
}

impl LogSettings {
    pub fn init_log(self) -> Result<LogGuard, Error> {
        let nonblocking_worker_guard = match self.output {
            LogOutput::Stdout => {
                let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
                self.init_subscriber(non_blocking)?;

                guard
            }
            LogOutput::Stderr => {
                let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stderr());
                self.init_subscriber(non_blocking)?;

                guard
            }
            LogOutput::File(ref path) => {
                let file = fs::OpenOptions::new()
                    .create(true)
                    
                    .append(true)
                    .open(path)
                    .map_err(|cause| Error::File {
                        path: path.clone(),
                        cause,
                    })?;
                let (non_blocking, guard) = tracing_appender::non_blocking(file);

                self.init_subscriber(non_blocking)?;

                guard
            }
        };

        let default_settings = Self::default();

        if self.output != default_settings.output {
            tracing::info!(
                "log output overriden from command line: {:?} replaced with {:?}",
                default_settings.output,
                self.output
            );
        }
        if self.level != default_settings.level {
            tracing::info!(
                "log level overriden from command line: {:?} replaced with {:?}",
                default_settings.level,
                self.level
            );
        }
        if self.format != default_settings.format {
            tracing::info!(
                "log format overriden from command line: {:?} replaced with {:?}",
                default_settings.format,
                self.format
            );
        }

        Ok(LogGuard {
            _nonblocking_worker_guard: nonblocking_worker_guard,
        })
    }

    fn init_subscriber(
        &self,
        non_blocking: tracing_appender::non_blocking::NonBlocking,
    ) -> Result<(), Error> {
        let otel_layer = if let Some(endpoint) = self.trace_collector_endpoint.as_ref() {
            let otel_tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint(endpoint.as_str()),
                )
                .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
                    opentelemetry::sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                        "explorer",
                    )]),
                ))
                .install_batch(opentelemetry::runtime::Tokio)
                .map_err(Error::InstallOpenTelemetryPipeLine)?;

            Some(tracing_opentelemetry::layer().with_tracer(otel_tracer))
        } else {
            None
        };

        let subscriber = tracing_subscriber::registry()
            .with(self.level)
            .with(otel_layer);

        match self.format {
            LogFormat::Plain => {
                let layer = tracing_subscriber::fmt::Layer::new()
                    .with_level(true)
                    .with_writer(non_blocking);

                subscriber
                    .with(layer)
                    .try_init()
                    .map_err(Error::InitSubscriber)
            }
            LogFormat::Json => {
                let layer = tracing_subscriber::fmt::Layer::new()
                    .json()
                    .with_level(true)
                    .with_writer(non_blocking);

                subscriber
                    .with(layer)
                    .try_init()
                    .map_err(Error::InitSubscriber)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("log format `{specified}` is not supported for this output")]
    FormatNotSupported { specified: LogFormat },
    #[error("failed to open the log file `{}`", .path.to_string_lossy())]
    File {
        path: PathBuf,
        #[source]
        cause: io::Error,
    },
    #[error("failed to install opentelemetry pipeline")]
    InstallOpenTelemetryPipeLine(#[source] opentelemetry::trace::TraceError),
    #[error("failed to init subscriber")]
    InitSubscriber(#[source] tracing_subscriber::util::TryInitError),
}
