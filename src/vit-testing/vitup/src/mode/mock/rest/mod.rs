mod admin;
mod control;
mod cors;
mod node;
pub mod reject;
mod search;
mod ssl;
mod vit_ss;

use futures::StreamExt;
use rustls::KeyLogFile;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tracing::{span, Level};
use warp::hyper::service::make_service_fn;

use super::ContextLock;
pub use admin::admin_filter;
use control::control_filter;
use cors::cors_filter;
use cors::default_headers;
use jormungandr_lib::interfaces::VotePlanId;
use node::*;
pub use ssl::{load_cert, load_private_key};
use thiserror::Error;
use valgrind::Protocol;
use vit_ss::*;
use warp::{reject::Reject, Filter, Rejection, Reply};

use reject::report_invalid;
impl Reject for crate::error::Error {}

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
}

impl Reject for Error {}

pub async fn start_rest_server(context: ContextLock) -> Result<(), Error> {
    let address = *context.read().unwrap().address();
    let protocol = context.read().unwrap().protocol();
    let with_context = context.clone();
    let with_context = warp::any().map(move || with_context.clone());

    let span = span!(Level::INFO, "rest");
    let _enter = span.enter();

    let root = warp::path!("api" / ..);

    let control_root = warp::path!("control" / ..).boxed();
    let control = control_filter(context.clone(), control_root).await;

    let health = warp::path!("health")
        .and(warp::post())
        .and_then(health_handler)
        .boxed();

    let v0 = {
        let root = warp::path!("v0" / ..);

        let admin_filter = admin_filter(context.clone());

        let proposals = {
            let root = warp::path!("proposals" / ..);
            let from_id = warp::path!(i32)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_proposal)
                .boxed();

            let from_idx = warp::path::end()
                .and(warp::post())
                .and(warp::body::json())
                .and(with_context.clone())
                .and_then(get_proposal_by_idx)
                .with(warp::reply::with::headers(default_headers()))
                .boxed();

            let proposals = warp::path!(String)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_all_proposals)
                .with(warp::reply::with::headers(default_headers()))
                .boxed();

            root.and(from_id.or(from_idx).or(proposals)).boxed()
        };

        let challenges = {
            let root = warp::path!("challenges" / ..);

            let challenges = warp::path::end()
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_challenges)
                .with(warp::reply::with::headers(default_headers()));

            let challenge_by_id = warp::path!(i32)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_challenge_by_id)
                .with(warp::reply::with::headers(default_headers()));

            let challenge_by_id_and_group_id = warp::path!(i32 / String)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_challenge_by_id_and_group_id);

            root.and(challenge_by_id_and_group_id.or(challenge_by_id.or(challenges)))
        };

        let reviews = {
            let root = warp::path!("reviews" / ..);

            let review_by_id = warp::path!(i32)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_review_by_id)
                .with(warp::reply::with::headers(default_headers()));

            root.and(review_by_id)
        };

        let funds = {
            let root = warp::path!("fund" / ..);

            let fund = warp::path::end()
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fund)
                .with(warp::reply::with::headers(default_headers()))
                .boxed();

            let fund_by_id = warp::path!(i32)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fund_by_id)
                .with(warp::reply::with::headers(default_headers()))
                .boxed();

            let all_funds = warp::path!("funds")
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_all_funds);

            root.and(fund_by_id.or(fund)).or(all_funds).boxed()
        };

        let settings = warp::path!("settings")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_settings)
            .with(warp::reply::with::headers(default_headers()))
            .boxed();

        let stats = warp::path!("node" / "stats")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_node_stats)
            .with(warp::reply::with::headers(default_headers()))
            .boxed();

        let account = warp::path!("account" / String)
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_account)
            .with(warp::reply::with::headers(default_headers()))
            .boxed();

        let fragment = {
            let root = warp::path!("fragment" / ..);

            let logs = warp::path!("logs")
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fragment_logs)
                .with(warp::reply::with::headers(default_headers()))
                .boxed();

            let debug = warp::path!("debug" / String)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(debug_message)
                .with(warp::reply::with::headers(default_headers()))
                .boxed();

            root.and(logs.or(debug)).boxed()
        };

        let message = warp::path!("message")
            .and(warp::post())
            .and(warp::body::bytes())
            .and(with_context.clone())
            .and_then(post_message)
            .with(warp::reply::with::headers(default_headers()))
            .boxed();

        let votes = warp::path!("vote" / "active" / "plans")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_active_vote_plans)
            .with(warp::reply::with::headers(default_headers()))
            .boxed();

        let block0 = warp::path!("block0")
            .and(with_context.clone())
            .map(move |context: ContextLock| {
                let span = span!(Level::INFO, "rest api call received");
                let _enter = span.enter();
                tracing::info!("get block0");
                //unwrapping is ok because this is a test module
                let context_result = context.read().unwrap();
                context_result.block0_bin()
            })
            .with(warp::reply::with::headers(default_headers()));

        let snapshot = {
            let root = warp::path!("snapshot" / ..);

            let get_voters_info = warp::path!("voter" / String / String)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_voters_info);

            let get_delegator_info = warp::path!("delegator" / String / String)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_delegator_info);

            let get_tags = warp::path::end()
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_tags);

            root.and(get_voters_info.or(get_delegator_info).or(get_tags))
        };

        let search = warp::path!("search")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context.clone())
            .and_then(search::search);

        let search_count = warp::path!("search_count")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context.clone())
            .and_then(search::search_count);

        root.and(
            proposals
                .or(admin_filter)
                .or(challenges)
                .or(funds)
                .or(reviews)
                .or(block0)
                .or(settings)
                .or(stats)
                .or(account)
                .or(fragment)
                .or(votes)
                .or(message)
                .or(snapshot)
                .or(search)
                .or(search_count),
        )
        .boxed()
    };

    let v1 = {
        let root = warp::path!("v1" / ..);

        let fragments = {
            let root = warp::path!("fragments" / ..);

            let post = warp::path::end()
                .and(warp::post())
                .and(warp::body::json())
                .and(with_context.clone())
                .and_then(post_fragments)
                .boxed();

            let status = warp::path!("statuses")
                .and(warp::get())
                .and(warp::query())
                .and(with_context.clone())
                .and_then(get_fragment_statuses)
                .with(warp::reply::with::headers(default_headers()))
                .boxed();

            let logs = warp::path!("logs")
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fragment_logs)
                .with(warp::reply::with::headers(default_headers()))
                .boxed();

            root.and(post.or(status).or(logs)).boxed()
        };

        let votes_with_plan = warp::path!("votes" / "plan" / VotePlanId / "account-votes" / String)
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_account_votes_with_plan)
            .with(warp::reply::with::headers(default_headers()));

        let votes = warp::path!("votes" / "plan" / "account-votes" / String)
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_account_votes)
            .with(warp::reply::with::headers(default_headers()));

        root.and(fragments.or(votes).or(votes_with_plan))
    };

    let version = warp::path!("vit-version")
        .and(warp::get())
        .and(with_context.clone())
        .map(move |context: ContextLock| warp::reply::json(&context.read().unwrap().version()))
        .with(warp::reply::with::headers(default_headers()));

    let cors_filter = cors_filter();

    let api = root
        .and(health.or(control).or(v0).or(v1).or(version))
        .recover(report_invalid)
        .with(cors_filter);

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

            tracing::info!("serving at: https://{}", address);
            Ok(server.await?)
        }
        Protocol::Http => {
            tracing::info!("serving at: http://{}", address);
            warp::serve(api).bind(address).await;
            Ok(())
        }
    }
}

pub async fn health_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply())
}
