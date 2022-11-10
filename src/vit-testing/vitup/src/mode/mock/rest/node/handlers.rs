use ::function_name::named;

use crate::mode::mock::rest::reject::{ForcedErrorCode, GeneralException, InvalidBatch};
use crate::mode::mock::ContextLock;
use chain_core::property::{Deserialize, Fragment as _};
use chain_crypto::PublicKey;
use chain_impl_mockchain::account;
use chain_impl_mockchain::account::{AccountAlg, Identifier};
use chain_impl_mockchain::fragment::{Fragment, FragmentId};
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::interfaces::{AccountVotes, FragmentsBatch, VotePlanId, VotePlanStatus};
use std::str::FromStr;
use vit_servicing_station_lib::v0::errors::HandleError;
use vit_servicing_station_lib::v0::result::HandlerResult;
use warp::{Rejection, Reply};

pub async fn post_message(
    message: warp::hyper::body::Bytes,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context = context.lock().unwrap();

    let fragment =
        match Fragment::deserialize(&mut chain_core::packer::Codec::new(message.as_ref())) {
            Ok(fragment) => fragment,
            Err(err) => {
                let code = context.state().error_code;
                context.log(format!(
                    "post_message with wrong fragment. Reason '{:?}'... Error code: {}",
                    err, code
                ));
                return Err(warp::reject::custom(ForcedErrorCode { code }));
            }
        };

    context.log(format!("post_message {}...", fragment.id()));

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    let fragment_id: jormungandr_lib::crypto::hash::Hash =
        context.state_mut().ledger_mut().message(fragment).into();
    Ok(HandlerResult(Ok(fragment_id)))
}

#[named]
pub async fn get_node_stats(context: ContextLock) -> Result<impl Reply, Rejection> {
    let mut context = context.lock().unwrap();
    context.log(function_name!());

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    Ok(HandlerResult(Ok(context.state().node_stats())))
}

#[named]
pub async fn get_settings(context: ContextLock) -> Result<impl Reply, Rejection> {
    let mut context = context.lock().unwrap();

    context.log(function_name!());

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    let settings = context.state().ledger().settings();

    Ok(HandlerResult(Ok(settings)))
}

fn parse_account_id(id_hex: &str) -> Result<account::Identifier, Rejection> {
    PublicKey::<AccountAlg>::from_str(id_hex)
        .map(Into::into)
        .map_err(|_| warp::reject::custom(GeneralException::hex_encoding_malformed()))
}

pub async fn get_account(
    account_bech32: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context = context.lock().unwrap();

    context.log(format!("get_account {}...", &account_bech32));

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    {
        let state = context.state_mut();

        if state.block_account_endpoint() != 0 {
            state.decrement_block_account_endpoint();
            let code = state.error_code;
            context.log(&format!(
                "block account endpoint mode is on. Rejecting with error code: {}",
                code
            ));
            return Err(warp::reject::custom(ForcedErrorCode { code }));
        }
    }

    let account_state: jormungandr_lib::interfaces::AccountState = context
        .state()
        .ledger()
        .accounts()
        .get_state(&parse_account_id(&account_bech32)?)
        .map(Into::into)
        .map_err(|_| warp::reject::custom(GeneralException::account_does_not_exist()))?;

    Ok(HandlerResult(Ok(account_state)))
}

#[derive(serde::Deserialize, Debug)]
pub struct GetMessageStatusesQuery {
    fragment_ids: String,
}

impl GetMessageStatusesQuery {
    pub fn as_fragment_ids(&self) -> Result<Vec<FragmentId>, super::super::Error> {
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
    let mut context = context.lock().unwrap();

    context.log(format!("get_fragment_statuses {:?}...", query));

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    let ids = query.as_fragment_ids();
    if let Err(err) = ids {
        return Err(warp::reject::custom(err));
    }

    Ok(HandlerResult(Ok(context
        .state()
        .ledger()
        .statuses(ids.unwrap()))))
}

#[named]
pub async fn post_fragments(
    batch: FragmentsBatch,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context = context.lock().unwrap();

    context.log(function_name!());

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

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

#[named]
pub async fn get_fragment_logs(context: ContextLock) -> Result<impl Reply, Rejection> {
    let mut context = context.lock().unwrap();

    context.log(function_name!());

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    Ok(HandlerResult(Ok(context.state().ledger().fragment_logs())))
}

pub async fn get_account_votes_with_plan(
    vote_plan_id: VotePlanId,
    acccount_id_hex: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context = context.lock().unwrap();

    context.log(format!(
        "get_account_votes: vote plan id {:?}. acccount id hex: {:?}",
        vote_plan_id, acccount_id_hex
    ));

    let identifier = into_identifier(acccount_id_hex)?;

    let vote_plan_id: chain_crypto::digest::DigestOf<_, _> = vote_plan_id.into_digest().into();

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    let maybe_vote_plan = context
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

    if let Some(error_code) = context_lock.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
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

pub fn into_identifier(account_id_hex: String) -> Result<Identifier, Rejection> {
    parse_account_id(&account_id_hex).map_err(|err| {
        warp::reject::custom(GeneralException {
            summary: format!("Cannot parse account id, due to: {:?}", err),
            code: 400,
        })
    })
}

#[named]
pub async fn get_active_vote_plans(context: ContextLock) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    context_lock.log(function_name!());

    if let Some(error_code) = context_lock.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
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
    let context = context.lock().unwrap();

    let id = FragmentId::from_str(&fragment_id).map_err(|_| HandleError::NotFound(fragment_id))?;
    let fragments = context.state().ledger().received_fragments();
    let fragment = fragments
        .iter()
        .find(|x| x.id() == id)
        .ok_or_else(|| HandleError::NotFound(id.to_string()))?;

    Ok(HandlerResult(Ok(format!("{:?}", fragment))))
}
