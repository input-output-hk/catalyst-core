use crate::config::{Config, SnapshotInitials};
use crate::mode::mock::rest::reject::GeneralException;
use crate::mode::mock::{ContextLock, FragmentRecieveStrategy, LedgerState, NetworkCongestionMode};
use crate::mode::service::manager::file_lister::dump_json;
use jortestkit::web::api_token::{APIToken, APITokenManager, TokenError};
use tracing::{info, trace};
use vit_servicing_station_lib::db::models::funds::Fund;
use vit_servicing_station_lib::v0::result::HandlerResult;
use warp::{Rejection, Reply};

#[tracing::instrument(skip(context), name = "control")]
pub async fn file_lister_handler(context: ContextLock) -> Result<impl Reply, Rejection> {
    info!("get files command");
    let context_lock = context.read().unwrap();
    Ok(dump_json(context_lock.working_dir())?).map(|r| warp::reply::json(&r))
}

#[tracing::instrument(skip(context, config), name = "mock control command received")]
pub async fn command_reset_mock(
    context: ContextLock,
    config: Config,
) -> Result<impl Reply, Rejection> {
    info!("reset voting command received");
    context.write().unwrap().reset(config)?;
    Ok(warp::reply())
}

#[tracing::instrument(skip(context), name = "mock control command received")]
pub async fn command_available(
    available: bool,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    info!("set mock availability command");
    context.write().unwrap().state_mut().available = available;
    Ok(warp::reply())
}

#[tracing::instrument(skip(context), name = "mock control command received")]
pub async fn command_block_account(
    block_account_endpoint_counter: u32,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    info!("block account command");
    context
        .write()
        .unwrap()
        .state_mut()
        .set_block_account_endpoint(block_account_endpoint_counter);
    Ok(warp::reply())
}

#[tracing::instrument(skip(context), name = "mock control command received")]
pub async fn command_reset_block_account(context: ContextLock) -> Result<impl Reply, Rejection> {
    info!("reset block account command");
    context
        .write()
        .unwrap()
        .state_mut()
        .reset_block_account_endpoint();
    Ok(warp::reply())
}

#[tracing::instrument(skip(ledger_state), name = "mock control command received")]
pub fn update_fragment_id(
    fragment_id: String,
    ledger_state: &mut LedgerState,
    fragment_strategy: FragmentRecieveStrategy,
) -> Result<impl Reply, Rejection> {
    info!("update fragment id command");
    if fragment_id.to_uppercase() == *"LAST".to_string() {
        trace!("updating last fragment to {}", fragment_strategy);
        let _ = ledger_state.set_status_for_recent_fragment(fragment_strategy);
    } else {
        trace!(
            "updating fragment with id '{}' to be: {}",
            fragment_id,
            fragment_strategy
        );
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
    let mut context_lock = context.write().unwrap();
    let ledger = context_lock.state_mut().ledger_mut();
    update_fragment_id(fragment_id, ledger, FragmentRecieveStrategy::Forget)
}

pub async fn command_update_reject(
    fragment_id: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.write().unwrap();
    let ledger = context_lock.state_mut().ledger_mut();
    update_fragment_id(fragment_id, ledger, FragmentRecieveStrategy::Reject)
}

pub async fn command_update_accept(
    fragment_id: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.write().unwrap();
    let ledger = context_lock.state_mut().ledger_mut();
    update_fragment_id(fragment_id, ledger, FragmentRecieveStrategy::Accept)
}

pub async fn command_update_pending(
    fragment_id: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.write().unwrap();
    let ledger = context_lock.state_mut().ledger_mut();
    update_fragment_id(fragment_id, ledger, FragmentRecieveStrategy::Pending)
}

#[tracing::instrument(skip(context), name = "mock control command received")]
pub async fn command_error_code(
    error_code: u16,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    info!("always return error code for REST Api call");
    let mut context_lock = context.write().unwrap();
    context_lock.state_mut().error_code = error_code;
    Ok(warp::reply())
}

pub async fn command_fund_id(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    context.write().unwrap().state_mut().set_fund_id(id);
    Ok(warp::reply())
}

pub async fn command_update_fund(
    fund: Fund,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context.write().unwrap().state_mut().update_fund(fund);
    Ok(warp::reply())
}

pub async fn command_version(
    version: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    context.write().unwrap().state_mut().set_version(version);
    Ok(warp::reply())
}

pub async fn command_congestion_normal(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .write()
        .unwrap()
        .state_mut()
        .set_congestion(NetworkCongestionMode::Normal);
    Ok(warp::reply())
}

pub async fn command_congestion_jammed(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .write()
        .unwrap()
        .state_mut()
        .set_congestion(NetworkCongestionMode::Jammed);
    Ok(warp::reply())
}

pub async fn command_congestion_moderate(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .write()
        .unwrap()
        .state_mut()
        .set_congestion(NetworkCongestionMode::Moderate);
    Ok(warp::reply())
}

pub async fn command_congestion_reset(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .write()
        .unwrap()
        .state_mut()
        .set_congestion(NetworkCongestionMode::Disabled);
    Ok(warp::reply())
}

pub async fn command_reject(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .write()
        .unwrap()
        .state_mut()
        .ledger_mut()
        .set_fragment_strategy(FragmentRecieveStrategy::Reject);
    Ok(warp::reply())
}

pub async fn command_accept(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .write()
        .unwrap()
        .state_mut()
        .ledger_mut()
        .set_fragment_strategy(FragmentRecieveStrategy::Accept);
    Ok(warp::reply())
}
pub async fn command_pending(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .write()
        .unwrap()
        .state_mut()
        .ledger_mut()
        .set_fragment_strategy(FragmentRecieveStrategy::Pending);
    Ok(warp::reply())
}
pub async fn command_reset(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .write()
        .unwrap()
        .state_mut()
        .ledger_mut()
        .set_fragment_strategy(FragmentRecieveStrategy::None);
    Ok(warp::reply())
}

pub async fn command_forget(context: ContextLock) -> Result<impl Reply, Rejection> {
    context
        .write()
        .unwrap()
        .state_mut()
        .ledger_mut()
        .set_fragment_strategy(FragmentRecieveStrategy::Forget);
    Ok(warp::reply())
}

pub async fn command_create_snapshot(
    config: SnapshotInitials,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let context_lock = context.read().unwrap();
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

pub async fn authorize_token(token: String, context: ContextLock) -> Result<(), Rejection> {
    let context = context.read().unwrap();
    let api_token = APIToken::from_string(token).map_err(warp::reject::custom)?;

    if context.api_token().is_none() {
        return Ok(());
    }

    let manager =
        APITokenManager::new(context.api_token().unwrap()).map_err(warp::reject::custom)?;

    if !manager.is_token_valid(api_token) {
        return Err(warp::reject::custom(TokenError::UnauthorizedToken));
    }
    Ok(())
}
