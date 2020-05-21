use crate::settings::{Cors, ServiceSettings, Tls};

use std::time::Duration;
use warp::filters::cors::Builder as CorsBuilder;
use warp::{Filter, TlsServer};

fn setup_cors(cors_config: Cors) -> CorsBuilder {
    let allowed_origins: Vec<&str> = cors_config
        .allowed_origins
        .iter()
        .map(AsRef::as_ref)
        .collect();

    let mut cors: CorsBuilder = warp::cors().allow_origins(allowed_origins);

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
    let server = warp::serve(app)
        .tls()
        .cert_path(tls_config.cert_file)
        .key_path(tls_config.priv_key_file);
    server
}

async fn start_server_with_config<App>(app: App, settings: ServiceSettings)
where
    App: Filter<Error = warp::Rejection> + Clone + Send + Sync + 'static,
    App::Extract: warp::Reply,
{
    let app = if let Some(cors) = settings.cors.map(setup_cors) {
        app.with(cors)
    } else {
        app.with(warp::cors())
    };

    if let Some(tls) = settings.tls {
        setup_tls(app, tls).bind(settings.address).await
    } else {
        warp::serve(app).bind(settings.address).await
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
        warp::serve(app).run(([127, 0, 0, 1], 3030)).await;
    }
}
