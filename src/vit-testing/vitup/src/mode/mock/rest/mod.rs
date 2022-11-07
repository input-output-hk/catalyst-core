use super::FragmentRecieveStrategy;
use super::{Context, ContextLock};
use crate::config::Config;
use crate::config::SnapshotInitials;
use crate::mode::mock::LedgerState;
use crate::mode::mock::NetworkCongestionMode;
use crate::mode::service::manager::file_lister::dump_json;
use chain_core::property::Deserialize as _;
use chain_core::property::Fragment as _;
use chain_crypto::PublicKey;
use chain_impl_mockchain::account::Identifier;
use chain_impl_mockchain::account::{self, AccountAlg};
use chain_impl_mockchain::fragment::{Fragment, FragmentId};
use futures::StreamExt;
use itertools::Itertools;
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::interfaces::AccountVotes;
use jormungandr_lib::interfaces::{FragmentsBatch, VotePlanId, VotePlanStatus};
use jortestkit::web::api_token::TokenError;
use jortestkit::web::api_token::{APIToken, APITokenManager, API_TOKEN_HEADER};
use rustls::KeyLogFile;
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use snapshot_lib::VoterHIR;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs::{self, File};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tracing_subscriber::fmt::format::FmtSpan;
use valgrind::Protocol;
use vit_servicing_station_lib::db::models::challenges::Challenge;
use vit_servicing_station_lib::db::models::funds::Fund;
use vit_servicing_station_lib::db::models::proposals::Proposal;
use vit_servicing_station_lib::db::queries::funds::{FundNextInfo, FundWithNext};
use vit_servicing_station_lib::v0::endpoints::proposals::ProposalsByVoteplanIdAndIndex;
use vit_servicing_station_lib::v0::errors::HandleError;
use vit_servicing_station_lib::v0::result::HandlerResult;
use warp::http::header::{HeaderMap, HeaderValue};
use warp::hyper::service::make_service_fn;
use warp::{reject::Reject, Filter, Rejection, Reply};
pub mod reject;
mod search;

use crate::error::Error::NoChallengeIdAndGroupFound;
use reject::{report_invalid, ForcedErrorCode, GeneralException, InvalidBatch};

impl warp::reject::Reject for crate::error::Error {}

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
    let is_token_enabled = context.lock().unwrap().api_token().is_some();
    let address = *context.lock().unwrap().address();
    let protocol = context.lock().unwrap().protocol();
    let working_dir = context.lock().unwrap().working_dir();
    let with_context = warp::any().map(move || context.clone());

    let (non_block, _guard) = tracing_appender::non_blocking(File::create("vole.trace").unwrap());

    let mut default_headers = HeaderMap::new();
    default_headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    default_headers.insert("vary", HeaderValue::from_static("Origin"));

    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());
    tracing_subscriber::fmt()
        .with_writer(non_block)
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let root = warp::path!("api" / ..);

    let control = {
        let root = warp::path!("control" / ..);

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

        let logs = {
            let root = warp::path!("logs" / ..).boxed();

            let list = warp::path!("get")
                .and(warp::get())
                .and(with_context.clone())
                .and_then(logs_get);

            let clear = warp::path!("clear")
                .and(warp::post())
                .and(with_context.clone())
                .and_then(logs_clear);

            root.and(clear.or(list)).boxed()
        };

        let files = {
            let root = warp::path!("files" / ..).boxed();

            let get = warp::path("get").and(warp::fs::dir(working_dir));
            let list = warp::path!("list")
                .and(warp::get())
                .and(with_context.clone())
                .and_then(file_lister_handler);

            root.and(get.or(list)).boxed()
        };

        let command = {
            let root = warp::path!("command" / ..);

            let reset = warp::path!("reset")
                .and(warp::post())
                .and(with_context.clone())
                .and(warp::body::json())
                .and_then(command_reset_mock);

            let availability = warp::path!("available" / bool)
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_available);

            let set_error_code = warp::path!("error-code" / u16)
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_error_code);

            let fund = {
                let root = warp::path!("fund");

                let fund_id = warp::path!("id" / i32)
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_fund_id);

                let fund_update = warp::path!("update")
                    .and(warp::put())
                    .and(warp::body::json())
                    .and(with_context.clone())
                    .and_then(command_update_fund);

                root.and(fund_id.or(fund_update))
            };

            let version = warp::path!("version" / String)
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_version);

            let block_account = {
                let root = warp::path!("block-account" / ..);

                let block_counter = warp::path!(u32)
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_block_account);

                let reset = warp::path!("reset")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_reset_block_account);

                root.and(block_counter.or(reset)).boxed()
            };

            let fragment_strategy = {
                let root = warp::path!("fragments" / ..);

                let reject = warp::path!("reject")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_reject);

                let accept = warp::path!("accept")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_accept);

                let pending = warp::path!("pending")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_pending);

                let reset = warp::path!("reset")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_reset);

                let forget = warp::path!("forget")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_forget);

                let update = {
                    let root = warp::path!("update" / ..);

                    let reject = warp::path!(String / "reject")
                        .and(warp::post())
                        .and(with_context.clone())
                        .and_then(command_update_reject);

                    let accept = warp::path!(String / "accept")
                        .and(warp::post())
                        .and(with_context.clone())
                        .and_then(command_update_accept);

                    let pending = warp::path!(String / "pending")
                        .and(warp::post())
                        .and(with_context.clone())
                        .and_then(command_update_pending);

                    let forget = warp::path!(String / "forget")
                        .and(warp::post())
                        .and(with_context.clone())
                        .and_then(command_update_forget);

                    root.and(reject.or(accept).or(pending).or(forget)).boxed()
                };

                root.and(
                    reject
                        .or(accept)
                        .or(pending)
                        .or(reset)
                        .or(update)
                        .or(forget)
                        .or(block_account),
                )
                .boxed()
            };

            let network_strategy = {
                let root = warp::path!("congestion" / ..);

                let normal = warp::path!("normal")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_congestion_normal);

                let jammed = warp::path!("jammed")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_congestion_jammed);

                let moderate = warp::path!("moderate")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_congestion_moderate);

                let reset = warp::path!("reset")
                    .and(warp::post())
                    .and(with_context.clone())
                    .and_then(command_congestion_reset);

                root.and(normal.or(jammed).or(moderate).or(reset)).boxed()
            };

            let snapshot_service = {
                let root = warp::path!("snapshot" / ..);

                let add = warp::path!("add" / String)
                    .and(warp::post())
                    .and(warp::body::json())
                    .and(with_context.clone())
                    .and_then(command_add_snapshot);

                let create = warp::path!("create")
                    .and(warp::post())
                    .and(warp::body::json())
                    .and(with_context.clone())
                    .and_then(command_create_snapshot);

                root.and(add.or(create)).boxed()
            };

            root.and(
                reset
                    .or(availability)
                    .or(set_error_code)
                    .or(fund)
                    .or(fragment_strategy)
                    .or(network_strategy)
                    .or(version)
                    .or(snapshot_service),
            )
            .boxed()
        };
        root.and(api_token_filter)
            .and(command.or(files).or(logs))
            .boxed()
    };

    let health = warp::path!("health")
        .and(warp::get())
        .and_then(health_handler)
        .boxed();

    let v0 = {
        let root = warp::path!("v0" / ..);

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
                .with(warp::reply::with::headers(default_headers.clone()))
                .boxed();

            let proposals = warp::path!(String)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_all_proposals)
                .with(warp::reply::with::headers(default_headers.clone()))
                .boxed();

            root.and(from_id.or(from_idx).or(proposals)).boxed()
        };

        let challenges = {
            let root = warp::path!("challenges" / ..);

            let challenges = warp::path::end()
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_challenges)
                .with(warp::reply::with::headers(default_headers.clone()));

            let challenge_by_id = warp::path!(i32)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_challenge_by_id)
                .with(warp::reply::with::headers(default_headers.clone()));

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
                .with(warp::reply::with::headers(default_headers.clone()));

            root.and(review_by_id)
        };

        let funds = {
            let root = warp::path!("fund" / ..);

            let fund = warp::path::end()
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fund)
                .with(warp::reply::with::headers(default_headers.clone()))
                .boxed();

            let fund_by_id = warp::path!(i32)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fund_by_id)
                .with(warp::reply::with::headers(default_headers.clone()))
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
            .with(warp::reply::with::headers(default_headers.clone()))
            .boxed();

        let stats = warp::path!("node" / "stats")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_node_stats)
            .with(warp::reply::with::headers(default_headers.clone()))
            .boxed();

        let account = warp::path!("account" / String)
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_account)
            .with(warp::reply::with::headers(default_headers.clone()))
            .boxed();

        let fragment = {
            let root = warp::path!("fragment" / ..);

            let logs = warp::path!("logs")
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fragment_logs)
                .with(warp::reply::with::headers(default_headers.clone()))
                .boxed();

            let debug = warp::path!("debug" / String)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(debug_message)
                .with(warp::reply::with::headers(default_headers.clone()))
                .boxed();

            root.and(logs.or(debug)).boxed()
        };

        let message = warp::path!("message")
            .and(warp::post())
            .and(warp::body::bytes())
            .and(with_context.clone())
            .and_then(post_message)
            .with(warp::reply::with::headers(default_headers.clone()))
            .boxed();

        let votes = warp::path!("vote" / "active" / "plans")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_active_vote_plans)
            .with(warp::reply::with::headers(default_headers.clone()))
            .boxed();

        let block0 = warp::path!("block0")
            .and(with_context.clone())
            .map(move |context: ContextLock| {
                context.lock().unwrap().log("get_block0");
                Ok(context.lock().unwrap().block0_bin())
            })
            .with(warp::reply::with::headers(default_headers.clone()));

        let snapshot = {
            let root = warp::path!("snapshot" / ..);

            let voting_power = warp::path!(String / String)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_voting_power)
                .boxed();

            let tags = warp::path::end()
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_tags)
                .boxed();

            let dump = warp::path!(String)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_snapshot)
                .boxed();

            root.and(tags.or(voting_power).or(dump))
        };

        let search = warp::path!("search")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_context.clone())
            .and_then(search::search);

        root.and(
            proposals
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
                .or(search),
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
                .with(warp::reply::with::headers(default_headers.clone()))
                .boxed();

            let logs = warp::path!("logs")
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fragment_logs)
                .with(warp::reply::with::headers(default_headers.clone()))
                .boxed();

            root.and(post.or(status).or(logs)).boxed()
        };

        let votes_with_plan = warp::path!("votes" / "plan" / VotePlanId / "account-votes" / String)
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_account_votes_with_plan)
            .with(warp::reply::with::headers(default_headers.clone()));

        let votes = warp::path!("votes" / "plan" / "account-votes" / String)
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_account_votes)
            .with(warp::reply::with::headers(default_headers.clone()));

        root.and(fragments.or(votes).or(votes_with_plan))
    };

    let version = warp::path!("vit-version")
        .and(warp::get())
        .and(with_context.clone())
        .map(move |context: ContextLock| warp::reply::json(&context.lock().unwrap().version()))
        .with(warp::reply::with::headers(default_headers.clone()));

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods((vec!["GET", "POST", "OPTIONS", "PUT", "PATCH"]).clone())
        .allow_headers(vec!["content-type"])
        .build();

    let api = root
        .and(health.or(control).or(v0).or(v1).or(version))
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

fn load_cert(filename: &Path) -> Result<Vec<rustls::Certificate>, Error> {
    let certfile = fs::File::open(filename)?;
    let mut reader = std::io::BufReader::new(certfile);

    match rustls_pemfile::read_one(&mut reader)? {
        Some(rustls_pemfile::Item::X509Certificate(cert)) => Ok(vec![rustls::Certificate(cert)]),
        Some(rustls_pemfile::Item::RSAKey(_)) | Some(rustls_pemfile::Item::PKCS8Key(_)) => {
            // TODO: a more specific error could be useful (ExpectedCertFoundKey?)
            Err(Error::InvalidCertificate)
        }
        // not a pemfile
        None => Err(Error::InvalidCertificate),
        Some(_) => Err(Error::InvalidCertificate),
    }
}

fn load_private_key(filename: &Path) -> Result<rustls::PrivateKey, Error> {
    let keyfile = File::open(filename)?;
    let mut reader = std::io::BufReader::new(keyfile);

    match rustls_pemfile::read_one(&mut reader)? {
        Some(rustls_pemfile::Item::RSAKey(key)) => Ok(rustls::PrivateKey(key)),
        Some(rustls_pemfile::Item::PKCS8Key(key)) => Ok(rustls::PrivateKey(key)),
        None | Some(rustls_pemfile::Item::X509Certificate(_)) => Err(Error::InvalidKey),
        Some(_) => Err(Error::InvalidCertificate),
    }
}

pub async fn get_account_votes_with_plan(
    vote_plan_id: VotePlanId,
    acccount_id_hex: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    context_lock.log(format!(
        "get_account_votes: vote plan id {:?}. acccount id hex: {:?}",
        vote_plan_id, acccount_id_hex
    ));

    let identifier = into_identifier(acccount_id_hex)?;

    let vote_plan_id: chain_crypto::digest::DigestOf<_, _> = vote_plan_id.into_digest().into();

    if !context_lock.available() {
        let code = context_lock.state().error_code;
        context_lock.log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let maybe_vote_plan = context_lock
        .state()
        .ledger()
        .active_vote_plans()
        .into_iter()
        .find(|x| x.id == vote_plan_id);
    let vote_plan = match maybe_vote_plan {
        Some(vote_plan) => vote_plan,
        None => {
            return Err(warp::reject::custom(GeneralException {
                summary: "".to_string(),
                code: 404,
            }))
        }
    };
    let result: Vec<u8> = vote_plan
        .proposals
        .into_iter()
        .enumerate()
        .filter(|(_, x)| x.votes.contains_key(&identifier))
        .map(|(i, _)| i as u8)
        .collect();

    Ok(HandlerResult(Ok(Some(result))))
}

pub async fn get_account_votes(
    account_id_hex: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    context_lock.log(format!(
        "get_account_votes: account id hex: {:?}",
        account_id_hex
    ));

    let identifier = into_identifier(account_id_hex)?;

    if !context_lock.available() {
        let code = context_lock.state().error_code;
        context_lock.log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let result: Vec<AccountVotes> = context_lock
        .state()
        .ledger()
        .active_vote_plans()
        .iter()
        .map(|vote_plan| {
            let votes: Vec<u8> = vote_plan
                .proposals
                .iter()
                .enumerate()
                .filter(|(_, x)| x.votes.contains_key(&identifier))
                .map(|(i, _)| i as u8)
                .collect();

            AccountVotes {
                vote_plan_id: Hash::from_str(&vote_plan.id.to_string()).unwrap(),
                votes,
            }
        })
        .collect();

    Ok(HandlerResult(Ok(Some(result))))
}

pub fn into_identifier(account_id_hex: String) -> Result<account::Identifier, Rejection> {
    parse_account_id(&account_id_hex).map_err(|err| {
        println!("{:?}", err);
        warp::reject::custom(GeneralException {
            summary: "Cannot parse account id".to_string(),
            code: 400,
        })
    })
}

pub async fn logs_get(context: ContextLock) -> Result<impl Reply, Rejection> {
    let context_lock = context.lock().unwrap();
    Ok(HandlerResult(Ok(context_lock.logs())))
}

pub async fn logs_clear(context: ContextLock) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    context_lock.clear_logs();
    Ok(warp::reply())
}

pub async fn file_lister_handler(context: ContextLock) -> Result<impl Reply, Rejection> {
    let context_lock = context.lock().unwrap();
    Ok(dump_json(context_lock.working_dir())?).map(|r| warp::reply::json(&r))
}

pub async fn get_active_vote_plans(context: ContextLock) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    context_lock.log("get_active_vote_plans");

    if !context_lock.available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let vp: Vec<VotePlanStatus> = context_lock
        .state()
        .ledger()
        .active_vote_plans()
        .into_iter()
        .map(VotePlanStatus::from)
        .collect();
    Ok(HandlerResult(Ok(vp)))
}

pub async fn debug_message(
    fragment_id: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let id = FragmentId::from_str(&fragment_id).map_err(|_| HandleError::NotFound(fragment_id))?;
    let fragments = context
        .lock()
        .unwrap()
        .state()
        .ledger()
        .received_fragments();
    let fragment = fragments
        .iter()
        .find(|x| x.id() == id)
        .ok_or_else(|| HandleError::NotFound(id.to_string()))?;

    Ok(HandlerResult(Ok(format!("{:?}", fragment))))
}

pub async fn command_reset_mock(
    context: ContextLock,
    config: Config,
) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().reset(config)?;
    Ok(warp::reply())
}

pub async fn command_available(
    available: bool,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().state_mut().available = available;
    Ok(warp::reply())
}

pub async fn command_block_account(
    block_account_endpoint_counter: u32,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .set_block_account_endpoint(block_account_endpoint_counter);
    Ok(warp::reply())
}

pub async fn command_reset_block_account(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .reset_block_account_endpoint();
    Ok(warp::reply())
}

fn update_fragment_id(
    fragment_id: String,
    ledger_state: &mut LedgerState,
    fragment_strategy: FragmentRecieveStrategy,
) -> Result<impl Reply, Rejection> {
    if fragment_id.to_uppercase() == *"LAST".to_string() {
        ledger_state.set_status_for_recent_fragment(fragment_strategy);
    } else {
        ledger_state
            .set_status_for_fragment_id(fragment_id, fragment_strategy)
            .map_err(|err| GeneralException {
                summary: err.to_string(),
                code: 404,
            })?;
    }
    Ok(warp::reply())
}

pub async fn command_update_forget(
    fragment_id: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    let ledger = context_lock.state_mut().ledger_mut();
    update_fragment_id(fragment_id, ledger, FragmentRecieveStrategy::Forget)
}

pub async fn command_update_reject(
    fragment_id: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    let ledger = context_lock.state_mut().ledger_mut();
    update_fragment_id(fragment_id, ledger, FragmentRecieveStrategy::Reject)
}

pub async fn command_update_accept(
    fragment_id: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    let ledger = context_lock.state_mut().ledger_mut();
    update_fragment_id(fragment_id, ledger, FragmentRecieveStrategy::Accept)
}

pub async fn command_update_pending(
    fragment_id: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    let ledger = context_lock.state_mut().ledger_mut();
    update_fragment_id(fragment_id, ledger, FragmentRecieveStrategy::Pending)
}

pub async fn command_error_code(
    error_code: u16,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .log(&format!("set-error-code: {}", error_code));
    context.lock().unwrap().state_mut().error_code = error_code;
    Ok(warp::reply())
}

pub async fn command_fund_id(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().state_mut().set_fund_id(id);
    Ok(warp::reply())
}

pub async fn command_update_fund(
    fund: Fund,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().state_mut().update_fund(fund);
    Ok(warp::reply())
}

pub async fn command_version(
    version: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().state_mut().set_version(version);
    Ok(warp::reply())
}

pub async fn command_congestion_normal(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .set_congestion(NetworkCongestionMode::Normal);
    Ok(warp::reply())
}

pub async fn command_congestion_jammed(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .set_congestion(NetworkCongestionMode::Jammed);
    Ok(warp::reply())
}

pub async fn command_congestion_moderate(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .set_congestion(NetworkCongestionMode::Moderate);
    Ok(warp::reply())
}

pub async fn command_congestion_reset(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .set_congestion(NetworkCongestionMode::Disabled);
    Ok(warp::reply())
}

pub async fn command_reject(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .ledger_mut()
        .set_fragment_strategy(FragmentRecieveStrategy::Reject);
    Ok(warp::reply())
}

pub async fn command_accept(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .ledger_mut()
        .set_fragment_strategy(FragmentRecieveStrategy::Accept);
    Ok(warp::reply())
}
pub async fn command_pending(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .ledger_mut()
        .set_fragment_strategy(FragmentRecieveStrategy::Pending);
    Ok(warp::reply())
}
pub async fn command_reset(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .ledger_mut()
        .set_fragment_strategy(FragmentRecieveStrategy::None);
    Ok(warp::reply())
}

pub async fn command_forget(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .ledger_mut()
        .set_fragment_strategy(FragmentRecieveStrategy::Forget);
    Ok(warp::reply())
}

async fn command_add_snapshot(
    tag: String,
    new_snapshot: Vec<VoterHIR>,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .voters_mut()
        .update_tag(tag, new_snapshot);
    Ok(warp::reply())
}

async fn command_create_snapshot(
    config: SnapshotInitials,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let context_lock = context.lock().unwrap();
    let state = context_lock.state();

    let voters_hirs = config
        .as_voters_hirs(state.defined_wallets())
        .map_err(|err| {
            warp::reject::custom(GeneralException {
                summary: err.to_string(),
                code: 500,
            })
        })?;

    Ok(HandlerResult(Ok(voters_hirs)))
}

pub async fn post_message(
    message: warp::hyper::body::Bytes,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let fragment =
        match Fragment::deserialize(&mut chain_core::packer::Codec::new(message.as_ref())) {
            Ok(fragment) => fragment,
            Err(err) => {
                let code = context.lock().unwrap().state().error_code;
                context.lock().unwrap().log(format!(
                    "post_message with wrong fragment. Reason '{:?}'... Error code: {}",
                    err, code
                ));
                return Err(warp::reject::custom(ForcedErrorCode { code }));
            }
        };

    context
        .lock()
        .unwrap()
        .log(format!("post_message {}...", fragment.id()));

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let fragment_id: jormungandr_lib::crypto::hash::Hash = context
        .lock()
        .unwrap()
        .state_mut()
        .ledger_mut()
        .message(fragment)
        .into();
    Ok(HandlerResult(Ok(fragment_id)))
}

#[derive(SerdeDeserialize, SerdeSerialize)]
pub struct ChallengeWithProposals {
    #[serde(flatten)]
    pub challenge: Challenge,
    pub proposals: Vec<Proposal>,
}

#[derive(serde::Deserialize, Debug)]
pub struct GetMessageStatusesQuery {
    fragment_ids: String,
}

impl GetMessageStatusesQuery {
    pub fn as_fragment_ids(&self) -> Result<Vec<FragmentId>, Error> {
        let ids = self.fragment_ids.split(',');
        ids.into_iter()
            .map(FragmentId::from_str)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
}

pub async fn get_fragment_statuses(
    query: GetMessageStatusesQuery,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .log(format!("get_fragment_statuses {:?}...", query));

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let ids = query.as_fragment_ids();
    if let Err(err) = ids {
        return Err(warp::reject::custom(err));
    }

    Ok(HandlerResult(Ok(context
        .lock()
        .unwrap()
        .state()
        .ledger()
        .statuses(ids.unwrap()))))
}

pub async fn post_fragments(
    batch: FragmentsBatch,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().log("post_fragments_v1");

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let mut context = context.lock().unwrap();
    let summary = context
        .state_mut()
        .ledger_mut()
        .batch_message(batch.fragments, batch.fail_fast);

    if !summary.rejected.is_empty() {
        Err(warp::reject::custom(InvalidBatch { summary, code: 400 }))
    } else {
        Ok(HandlerResult(Ok(summary)))
    }
}

pub async fn get_fragment_logs(context: ContextLock) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().log("get_fragment_logs");

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    Ok(HandlerResult(Ok(context
        .lock()
        .unwrap()
        .state()
        .ledger()
        .fragment_logs())))
}

pub async fn get_challenges(context: ContextLock) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().log("get_challenges");

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    Ok(HandlerResult(Ok(context
        .lock()
        .unwrap()
        .state()
        .vit()
        .challenges())))
}

pub async fn get_challenge_by_id_and_group_id(
    id: i32,
    voter_group_id: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    context_lock.log(format!(
        "get_challenge_by_id_and_group_id {}/{} ...",
        id, voter_group_id
    ));

    if !context_lock.available() {
        let code = context_lock.state().error_code;
        context_lock.log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let challenge = context_lock
        .state()
        .vit()
        .challenges()
        .iter()
        .cloned()
        .find(|ch| ch.id == id)
        .ok_or_else(|| HandleError::NotFound(id.to_string()))?;
    let proposals: Vec<Proposal> = context_lock
        .state()
        .vit()
        .proposals()
        .iter()
        .filter(|x| x.proposal.challenge_id == challenge.id && x.group_id == voter_group_id)
        .map(|x| x.proposal.clone())
        .collect();

    if proposals.is_empty() {
        Err(warp::reject::custom(NoChallengeIdAndGroupFound {
            id: id.to_string(),
            group: voter_group_id,
        }))
    } else {
        Ok(HandlerResult(Ok(ChallengeWithProposals {
            challenge,
            proposals,
        })))
    }
}

pub async fn get_challenge_by_id(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .log(format!("get_challenge_by_id {} ...", id));

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let challenge = context
        .lock()
        .unwrap()
        .state()
        .vit()
        .challenges()
        .iter()
        .cloned()
        .find(|ch| ch.id == id)
        .ok_or_else(|| HandleError::NotFound(id.to_string()))?;
    let proposals: Vec<Proposal> = context
        .lock()
        .unwrap()
        .state()
        .vit()
        .proposals()
        .iter()
        .filter(|x| x.proposal.challenge_id == challenge.id)
        .map(|x| x.proposal.clone())
        .collect();

    Ok(HandlerResult(Ok(ChallengeWithProposals {
        challenge,
        proposals,
    })))
}

pub async fn get_review_by_id(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .log(format!("get_review_by_id {} ...", id));

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let reviews: HashMap<String, _> = context
        .lock()
        .unwrap()
        .state()
        .vit()
        .advisor_reviews()
        .iter()
        .cloned()
        .filter(|review| review.proposal_id == id)
        .group_by(|review| review.assessor.to_string())
        .into_iter()
        .map(|(key, group)| (key, group.collect::<Vec<_>>()))
        .collect();

    Ok(HandlerResult(Ok(reviews)))
}

pub async fn get_all_proposals(
    voting_group: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().log("get_all_proposals");

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    Ok(HandlerResult(Ok(context
        .lock()
        .unwrap()
        .state()
        .vit()
        .proposals()
        .into_iter()
        .filter(|p| p.group_id == voting_group)
        .collect::<Vec<_>>())))
}

pub async fn get_proposal_by_idx(
    request: ProposalsByVoteplanIdAndIndex,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .log(format!("get_proposal_by_idx ({:?}) ...", request));

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let proposals: Vec<_> = context
        .lock()
        .unwrap()
        .state()
        .vit()
        .proposals()
        .iter()
        .filter(|x| {
            request.iter().any(|item| {
                x.voteplan.chain_voteplan_id == item.vote_plan_id
                    && item.indexes.contains(&x.voteplan.chain_proposal_index)
            })
        })
        .cloned()
        .collect();

    Ok(HandlerResult(Ok(proposals)))
}

pub async fn get_proposal(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .log(format!("get_proposal {} ...", id));

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let proposal = context
        .lock()
        .unwrap()
        .state()
        .vit()
        .proposals()
        .iter()
        .find(|x| x.proposal.internal_id.to_string() == id.to_string())
        .cloned()
        .ok_or_else(|| warp::reject::custom(GeneralException::proposal_not_found(id)))?;

    Ok(HandlerResult(Ok(proposal)))
}

pub async fn get_fund_by_id(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .log(format!("get_fund_by_id {} ...", id));

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let funds = context.lock().unwrap().state().vit().funds();

    let fund = funds.iter().find(|x| x.id == id).unwrap();

    Ok(HandlerResult(Ok(fund.clone())))
}

pub async fn get_fund(context: ContextLock) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().log("get_fund ...");

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let funds: Vec<Fund> = context.lock().unwrap().state().vit().funds().to_vec();
    let next = funds.get(1).map(|f| FundNextInfo {
        id: f.id,
        fund_name: f.fund_name.clone(),
        stage_dates: f.stage_dates.clone(),
    });
    let fund_with_next = FundWithNext {
        fund: funds.first().unwrap().clone(),
        next,
    };

    Ok(HandlerResult(Ok(fund_with_next)))
}

pub async fn get_all_funds(context: ContextLock) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().log("get_all_fund ...");

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let funds: Vec<Fund> = context.lock().unwrap().state().vit().funds().to_vec();

    Ok(warp::reply::json(&funds))
}

pub async fn get_node_stats(context: ContextLock) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().log("get_node_stats...");

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let context = context.lock().unwrap();

    Ok(HandlerResult(Ok(context.state().node_stats())))
}

pub async fn get_settings(context: ContextLock) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().log("get_settings...");

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    let settings = context.lock().unwrap().state().ledger().settings();

    Ok(HandlerResult(Ok(settings)))
}

fn parse_account_id(id_hex: &str) -> Result<Identifier, Rejection> {
    PublicKey::<AccountAlg>::from_str(id_hex)
        .map(Into::into)
        .map_err(|_| warp::reject::custom(GeneralException::hex_encoding_malformed()))
}

pub async fn get_account(
    account_bech32: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .log(format!("get_account {}...", &account_bech32));

    if !context.lock().unwrap().available() {
        let code = context.lock().unwrap().state().error_code;
        context.lock().unwrap().log(&format!(
            "unavailability mode is on. Rejecting with error code: {}",
            code
        ));
        return Err(warp::reject::custom(ForcedErrorCode { code }));
    }

    {
        let mut context_lock = context.lock().unwrap();
        let state = context_lock.state_mut();

        if state.block_account_endpoint() != 0 {
            state.decrement_block_account_endpoint();
            let code = state.error_code;
            context_lock.log(&format!(
                "block account endpoint mode is on. Rejecting with error code: {}",
                code
            ));
            return Err(warp::reject::custom(ForcedErrorCode { code }));
        }
    }

    let account_state: jormungandr_lib::interfaces::AccountState = context
        .lock()
        .unwrap()
        .state()
        .ledger()
        .accounts()
        .get_state(&parse_account_id(&account_bech32)?)
        .map(Into::into)
        .map_err(|_| warp::reject::custom(GeneralException::account_does_not_exist()))?;

    Ok(HandlerResult(Ok(account_state)))
}

pub async fn health_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply())
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

async fn get_voting_power(
    tag: String,
    key_hex: String,
    context: Arc<std::sync::Mutex<Context>>,
) -> Result<impl Reply, Rejection> {
    let entries = context
        .lock()
        .unwrap()
        .state()
        .voters()
        .get_voting_power(&tag, &parse_account_id(&key_hex)?.into())
        .into_iter()
        .map(
            |VoterHIR {
                 voting_group,
                 voting_power,
                 ..
             }| serde_json::json!({"voting_power": voting_power, "voting_group": voting_group}),
        )
        .collect::<Vec<_>>();
    Ok(warp::reply::json(&entries))
}

async fn get_snapshot(
    tag: String,
    context: Arc<std::sync::Mutex<Context>>,
) -> Result<impl Reply, Rejection> {
    let entries = context
        .lock()
        .unwrap()
        .state()
        .voters()
        .get_snapshot(&tag)
        .into_iter()
        .map(
            |VoterHIR {
                 voting_group,
                 voting_power,
                 voting_key,
             }| serde_json::json!({"voting_key": voting_key.to_hex(), "voting_power": voting_power, "voting_group": voting_group}),
        )
        .collect::<Vec<_>>();
    Ok(warp::reply::json(&entries))
}

async fn get_tags(context: Arc<std::sync::Mutex<Context>>) -> Result<impl Reply, Rejection> {
    let entries = context.lock().unwrap().state().voters().tags();
    Ok(warp::reply::json(&entries))
}
