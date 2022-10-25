use super::ContextError;
use super::{Context, ContextLock};
use crate::mode::mock::farm::context::MockId;
use crate::mode::mock::rest::reject::report_invalid;
use crate::mode::mock::rest::{load_cert, load_private_key};
use futures::StreamExt;
use hyper::service::make_service_fn;
use jortestkit::web::api_token::TokenError;
use jortestkit::web::api_token::{APIToken, APITokenManager, API_TOKEN_HEADER};
use rustls::KeyLogFile;
use std::convert::Infallible;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use valgrind::Protocol;
use vit_servicing_station_lib::v0::result::HandlerResult;
use warp::http::header::{HeaderMap, HeaderValue};
use warp::{reject::Reject, Filter, Rejection, Reply};

impl warp::reject::Reject for ContextError {}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse uuid")]
    CannotParseUuid(#[from] uuid::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Rusls(#[from] rustls::Error),
    #[error(transparent)]
    Crypto(#[from] chain_crypto::hash::Error),
    #[error("invalid tls certificate")]
    InvalidCertificate,
    #[error("invalid tls key")]
    InvalidKey,
    #[error(transparent)]
    Rest(#[from] crate::mode::mock::rest::Error),
}

impl Reject for Error {}

pub async fn start_rest_server(context: ContextLock) -> Result<(), Error> {
    let is_token_enabled = context.lock().unwrap().api_token().is_some();
    let address = context.lock().unwrap().address();
    let protocol = context.lock().unwrap().protocol();
    let with_context = warp::any().map(move || context.clone());

    let mut default_headers = HeaderMap::new();
    default_headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    default_headers.insert("vary", HeaderValue::from_static("Origin"));

    let root = warp::path!("api" / ..);

    let v0 = {
        let root = warp::path!("v0" / ..);

        let active = warp::path!("active")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_active_mocks)
            .boxed();

        let shutdown = warp::path!("shutdown" / String)
            .and(warp::post())
            .and(with_context.clone())
            .and_then(shutdown_mock)
            .with(warp::reply::with::headers(default_headers.clone()))
            .boxed();

        let start = {
            let root = warp::path!("start" / ..);

            let start_mock_on_random_port = warp::path!(String)
                .and(warp::post())
                .and(with_context.clone())
                .and_then(start_mock_on_random_port)
                .with(warp::reply::with::headers(default_headers.clone()))
                .boxed();

            root.and(start_mock_on_random_port).boxed()
        };

        root.and(active.or(shutdown).or(start)).boxed()
    };

    let api_token_filter = if is_token_enabled {
        warp::header::header(API_TOKEN_HEADER)
            .and(with_context.clone())
            .and_then(authorize_token)
            .and(warp::any())
            .untuple_one()
            .boxed()
    } else {
        warp::any().boxed()
    };

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods((vec!["GET", "POST", "OPTIONS", "PUT", "PATCH"]).clone())
        .allow_headers(vec!["content-type"])
        .build();

    let api = root
        .and(api_token_filter.and(v0))
        .recover(report_invalid)
        .with(cors);

    match protocol {
        Protocol::Https(certs) => {
            let tls_cfg = {
                let cert = load_cert(&certs.cert_path)?;
                let key = load_private_key(&certs.key_path)?;
                let mut cfg = rustls::ServerConfig::builder()
                    .with_safe_defaults()
                    .with_no_client_auth()
                    .with_single_cert(cert, key)?;

                cfg.key_log = Arc::new(KeyLogFile::new());
                cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
                Arc::new(cfg)
            };

            let tls_acceptor = TlsAcceptor::from(tls_cfg);
            let arc_acceptor = Arc::new(tls_acceptor);

            let listener =
                tokio_stream::wrappers::TcpListenerStream::new(TcpListener::bind(&address).await?);

            let incoming =
                hyper::server::accept::from_stream(listener.filter_map(|socket| async {
                    match socket {
                        Ok(stream) => match arc_acceptor.clone().accept(stream).await {
                            Ok(val) => Some(Ok::<_, hyper::Error>(val)),
                            Err(e) => {
                                tracing::warn!("handshake failed {}", e);
                                None
                            }
                        },
                        Err(e) => {
                            tracing::error!("tcp socket outer err: {}", e);
                            None
                        }
                    }
                }));

            let svc = warp::service(api);
            let service = make_service_fn(move |_| {
                let svc = svc.clone();
                async move { Ok::<_, Infallible>(svc) }
            });

            let server = hyper::Server::builder(incoming).serve(service);

            println!("serving at: https://{}", address);
            Ok(server.await?)
        }
        Protocol::Http => {
            println!("serving at: http://{}", address);
            warp::serve(api).bind(address).await;
            Ok(())
        }
    }
}

pub async fn get_active_mocks(context: ContextLock) -> Result<impl Reply, Rejection> {
    let context_lock = context.lock().unwrap();
    let active = context_lock.get_active_mocks();
    Ok(HandlerResult(Ok(Some(active))))
}

pub async fn shutdown_mock(id: MockId, context: ContextLock) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    context_lock.shutdown_mock(id.clone())?;
    Ok(HandlerResult(Ok(format!(
        "mock environment '{}' was shutdown successfully",
        id
    ))))
}

pub async fn start_mock_on_random_port(
    id: MockId,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    let port = context_lock.start_mock_on_random_port(id)?;
    Ok(HandlerResult(Ok(port)))
}

pub async fn authorize_token(
    token: String,
    context: Arc<std::sync::Mutex<Context>>,
) -> Result<(), Rejection> {
    let api_token = APIToken::from_string(token).map_err(warp::reject::custom)?;

    if context.lock().unwrap().api_token().is_none() {
        return Ok(());
    }

    let manager = APITokenManager::new(context.lock().unwrap().api_token().unwrap())
        .map_err(warp::reject::custom)?;

    if !manager.is_token_valid(api_token) {
        return Err(warp::reject::custom(TokenError::UnauthorizedToken));
    }
    Ok(())
}
