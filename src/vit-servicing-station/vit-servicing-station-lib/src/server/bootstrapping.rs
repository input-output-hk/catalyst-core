use super::settings::{Cors, ServiceSettings, Tls};

use std::time::Duration;
use warp::filters::cors::Builder as CorsBuilder;
use warp::{Filter, TlsServer};

fn setup_cors(cors_config: Cors) -> CorsBuilder {
    let mut cors: CorsBuilder = if let Some(allowed_origins) = cors_config.allowed_origins {
        let allowed_origins: Vec<&str> = allowed_origins.iter().map(AsRef::as_ref).collect();
        warp::cors().allow_origins(allowed_origins)
    } else {
        warp::cors().allow_any_origin()
    };

    if let Some(max_age) = cors_config.max_age_secs {
        cors = cors.max_age(Duration::from_secs(max_age));
    }
    cors
}

fn setup_tls<App>(app: App, tls_config: Tls) -> TlsServer<App>
where
    App: Filter<Error = warp::Rejection> + Clone + Send + Sync + 'static,
    App::Extract: warp::Reply,
{
    assert!(
        tls_config.is_loaded(),
        "Tls config should be filled before calling setup"
    );
    let (cert_file, priv_key_file) = (
        tls_config.cert_file.unwrap(),
        tls_config.priv_key_file.unwrap(),
    );
    warp::serve(app)
        .tls()
        .cert_path(cert_file)
        .key_path(priv_key_file)
}

async fn start_server_with_config<App>(app: App, settings: ServiceSettings)
where
    App: Filter<Error = warp::Rejection> + Clone + Send + Sync + 'static,
    App::Extract: warp::Reply,
{
    let app = app.with(setup_cors(settings.cors));

    if settings.tls.is_loaded() {
        let (_, server) = setup_tls(app, settings.tls).bind_with_graceful_shutdown(
            settings.address,
            super::signals::watch_signal_for_shutdown(),
        );
        server.await
    } else {
        let (_, server) = warp::serve(app).bind_with_graceful_shutdown(
            settings.address,
            super::signals::watch_signal_for_shutdown(),
        );
        server.await
    };
}

pub async fn start_server<App>(app: App, settings: Option<ServiceSettings>)
where
    App: Filter<Error = warp::Rejection> + Clone + Send + Sync + 'static,
    App::Extract: warp::Reply,
{
    if let Some(settings) = settings {
        start_server_with_config(app, settings).await
    } else {
        // easy way of starting a local debug server
        let (_, server) = warp::serve(app).bind_with_graceful_shutdown(
            ([127, 0, 0, 1], 3030),
            super::signals::watch_signal_for_shutdown(),
        );
        server.await
    }
}
