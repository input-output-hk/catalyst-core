use super::FragmentRecieveStrategy;
use crate::mock::context::{Context, ContextLock};
use chain_core::property::Deserialize;
use chain_crypto::PublicKey;
use chain_impl_mockchain::account::AccountAlg;
use chain_impl_mockchain::account::Identifier;
use jormungandr_lib::interfaces::VotePlanStatus;
use jortestkit::web::api_token::TokenError;
use jortestkit::web::api_token::{APIToken, APITokenManager, API_TOKEN_HEADER};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use std::convert::Infallible;
use std::fs::File;
use std::sync::Arc;
use thiserror::Error;
use tracing_subscriber::fmt::format::FmtSpan;
use vit_servicing_station_lib::db::models::challenges::Challenge;
use vit_servicing_station_lib::db::models::proposals::Proposal;
use vit_servicing_station_lib::v0::errors::HandleError;
use vit_servicing_station_lib::v0::result::HandlerResult;
use warp::{http::StatusCode, reject::Reject, Filter, Rejection, Reply};
impl Reject for crate::mock::context::Error {}
use crate::manager::file_lister::dump_json;
use crate::mock::context::Error::AccountDoesNotExist;
use chain_core::property::Fragment as _;
use chain_impl_mockchain::fragment::{Fragment, FragmentId};
use std::str::FromStr;
use vit_servicing_station_lib::db::models::funds::Fund;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse uuid")]
    CannotParseUuid(#[from] uuid::Error),
}

impl Reject for Error {}

pub async fn start_rest_server(context: ContextLock) {
    let is_token_enabled = context.lock().unwrap().api_token().is_some();
    let address = *context.lock().unwrap().address();
    let block0_content = context.lock().unwrap().block0_bin();
    let working_dir = context.lock().unwrap().working_dir();
    let with_context = warp::any().map(move || context.clone());

    let (non_block, _guard) = tracing_appender::non_blocking(File::create("vole.trace").unwrap());

    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_block)
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

            let availability = warp::path!("available" / bool)
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_available);

            let fund_id = warp::path!("fund" / "id" / i32)
                .and(warp::post())
                .and(with_context.clone())
                .and_then(command_fund_id);

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

                root.and(reject.or(accept).or(pending).or(reset)).boxed()
            };

            root.and(availability.or(fund_id).or(fragment_strategy))
                .boxed()
        };
        root.and(api_token_filter).and(command.or(files)).boxed()
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

            let proposals = warp::path::end()
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_all_proposals)
                .boxed();

            root.and(from_id.or(proposals)).boxed()
        };

        let challenges = {
            let root = warp::path!("challenges" / ..);

            let challenges = warp::path::end()
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_challenges);

            let challenge_by_id = warp::path!(i32)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_challenge_by_id);

            root.and(challenge_by_id.or(challenges))
        };

        let funds = {
            let root = warp::path!("fund" / ..);

            let fund = warp::path::end()
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fund)
                .boxed();

            let fund_by_id = warp::path!(i32)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fund_by_id)
                .boxed();

            root.and(fund_by_id.or(fund)).boxed()
        };

        let settings = warp::path!("settings")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_settings)
            .boxed();

        let account = warp::path!("account" / String)
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_account)
            .boxed();

        let fragment = {
            let root = warp::path!("fragment" / ..);

            let logs = warp::path!("logs")
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fragment_logs)
                .boxed();

            let debug = warp::path!("debug" / String)
                .and(warp::get())
                .and(with_context.clone())
                .and_then(debug_message)
                .boxed();

            root.and(logs.or(debug)).boxed()
        };

        let message = warp::path!("message")
            .and(warp::post())
            .and(warp::body::bytes())
            .and(with_context.clone())
            .and_then(post_message)
            .boxed();

        let votes = warp::path!("vote" / "active" / "plans")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(get_active_vote_plans)
            .boxed();

        let block0 = warp::path!("block0").map(move || {
            println!("get_block0 ...");
            Ok(block0_content.clone())
        });

        root.and(
            proposals
                .or(challenges)
                .or(funds)
                .or(block0)
                .or(settings)
                .or(account)
                .or(fragment)
                .or(votes)
                .or(message),
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
                .boxed();

            let logs = warp::path!("logs")
                .and(warp::get())
                .and(with_context.clone())
                .and_then(get_fragment_logs)
                .boxed();

            root.and(post.or(status).or(logs)).boxed()
        };

        root.and(fragments)
    };

    let api = root
        .and(health.or(control).or(v0).or(v1))
        .recover(report_invalid)
        .boxed();

    let server = warp::serve(api);

    let server_fut = server.bind(address);
    server_fut.await;
}

pub async fn file_lister_handler(context: ContextLock) -> Result<impl Reply, Rejection> {
    let context_lock = context.lock().unwrap();
    Ok(dump_json(context_lock.working_dir())?).map(|r| warp::reply::json(&r))
}

pub async fn get_active_vote_plans(context: ContextLock) -> Result<impl Reply, Rejection> {
    println!("get_active_vote_plans ...");

    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
    }

    let vp: Vec<VotePlanStatus> = context
        .lock()
        .unwrap()
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

pub async fn command_available(
    available: bool,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context.lock().unwrap().state_mut().available = available;
    Ok(warp::reply())
}

pub async fn command_fund_id(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .lock()
        .unwrap()
        .state_mut()
        .vit_mut()
        .funds_mut()
        .last_mut()
        .unwrap()
        .id = id;
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

pub async fn post_message(
    message: warp::hyper::body::Bytes,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let fragment = match Fragment::deserialize(message.as_ref()) {
        Ok(fragment) => fragment,
        Err(err) => {
            println!("post_message with wrong fragment reason '{:?}'...", err);
            return Err(warp::reject());
        }
    };
    println!("post_message {}...", fragment.id());

    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
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

#[derive(serde::Deserialize)]
pub struct GetMessageStatusesQuery {
    fragment_ids: String,
}

pub async fn get_fragment_statuses(
    query: GetMessageStatusesQuery,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    println!("get_fragment_statuses...");

    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
    }

    let ids = query.fragment_ids.split(',');
    let ids = ids
        .into_iter()
        .map(|s| FragmentId::from_str(s))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    Ok(HandlerResult(Ok(context
        .lock()
        .unwrap()
        .state()
        .ledger()
        .statuses(ids))))
}

pub async fn post_fragments(
    messages: Vec<String>,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    println!("post_fragments...");
    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
    }

    let fragments: Vec<Fragment> = messages
        .into_iter()
        .map(|message| {
            let message = hex::decode(message).unwrap();
            Fragment::deserialize(message.as_slice()).unwrap()
        })
        .collect();
    let fragment_ids: Vec<String> = fragments
        .iter()
        .map(|fragment| fragment.id().to_string())
        .collect();

    let mut context = context.lock().unwrap();

    for fragment in fragments.into_iter() {
        let _ = context.state_mut().ledger_mut().message(fragment);
    }
    Ok(HandlerResult(Ok(fragment_ids)))
}

pub async fn get_fragment_logs(context: ContextLock) -> Result<impl Reply, Rejection> {
    println!("get_fragment_logs...");
    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
    }

    Ok(HandlerResult(Ok(context
        .lock()
        .unwrap()
        .state()
        .ledger()
        .fragment_logs())))
}

pub async fn get_challenges(context: ContextLock) -> Result<impl Reply, Rejection> {
    println!("get_challenges...");
    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
    }

    Ok(HandlerResult(Ok(context
        .lock()
        .unwrap()
        .state()
        .vit()
        .challenges())))
}

pub async fn get_challenge_by_id(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    println!("get_challenge_by_id {} ...", id);

    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
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

pub async fn get_all_proposals(context: ContextLock) -> Result<impl Reply, Rejection> {
    println!("get_all_proposals...");

    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
    }

    Ok(HandlerResult(Ok(context
        .lock()
        .unwrap()
        .state()
        .vit()
        .proposals()
        .iter()
        .map(|x| x.proposal.clone())
        .collect::<Vec<Proposal>>())))
}

pub async fn get_proposal(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    println!("get_proposal {} ...", id);

    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
    }

    let proposal = context
        .lock()
        .unwrap()
        .state()
        .vit()
        .proposals()
        .iter()
        .find(|x| x.proposal.internal_id.to_string() == id.to_string())
        .map(|x| x.proposal.clone())
        .unwrap();

    Ok(HandlerResult(Ok(proposal)))
}

pub async fn get_fund_by_id(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    println!("get_fund_by_id {} ...", id);

    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
    }

    let funds = context.lock().unwrap().state().vit().funds();

    let fund = funds.iter().find(|x| x.id == id).unwrap();

    Ok(HandlerResult(Ok(fund.clone())))
}

pub async fn get_fund(context: ContextLock) -> Result<impl Reply, Rejection> {
    println!("get_fund ...");

    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
    }

    let funds: Vec<Fund> = context.lock().unwrap().state().vit().funds().to_vec();

    Ok(HandlerResult(Ok(funds.first().unwrap().clone())))
}

pub async fn get_settings(context: ContextLock) -> Result<impl Reply, Rejection> {
    println!("get_settings ...");

    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
    }

    let settings = context.lock().unwrap().state().ledger().settings();

    Ok(HandlerResult(Ok(settings)))
}

fn parse_account_id(id_hex: &str) -> Identifier {
    PublicKey::<AccountAlg>::from_str(id_hex)
        .map(Into::into)
        .unwrap()
}

pub async fn get_account(
    account_bech32: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    println!("get_account {}...", &account_bech32);

    if !context.lock().unwrap().available() {
        println!("unavailability mode is on");
        return Err(warp::reject());
    }
    let account_state: Result<jormungandr_lib::interfaces::AccountState, _> = context
        .lock()
        .unwrap()
        .state()
        .ledger()
        .accounts()
        .get_state(&parse_account_id(&account_bech32))
        .map(Into::into)
        .map_err(|_| AccountDoesNotExist);

    Ok(HandlerResult(Ok(account_state?)))
}

pub async fn health_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply())
}

async fn report_invalid(r: Rejection) -> Result<impl Reply, Infallible> {
    /* if let Some(e) = r.find::<file_lister::Error>() {
        Ok(warp::reply::with_status(
            e.to_string(),
            StatusCode::BAD_REQUEST,
        ))
    } else {*/
    Ok(warp::reply::with_status(
        format!("internal error: {:?}", r),
        StatusCode::INTERNAL_SERVER_ERROR,
    ))
    // }
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
