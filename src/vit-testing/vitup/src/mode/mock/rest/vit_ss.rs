use crate::error::Error::NoChallengeIdAndGroupFound;
use crate::mode::mock::rest::reject::GeneralException;
use crate::mode::mock::ContextLock;
use chain_impl_mockchain::value::Value;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Debug;
use tracing::info;
use vit_servicing_station_lib::db::models::funds::Fund;
use vit_servicing_station_lib::db::models::proposals::Proposal;
use vit_servicing_station_lib::db::queries::funds::{FundNextInfo, FundWithNext};
use vit_servicing_station_lib::v0::endpoints::challenges::ChallengeWithProposals;
use vit_servicing_station_lib::v0::endpoints::proposals::ProposalsByVoteplanIdAndIndex;
use vit_servicing_station_lib::v0::endpoints::snapshot::{DelegatorInfo, VoterInfo, VotersInfo};
use vit_servicing_station_lib::v0::errors::HandleError;
use vit_servicing_station_lib::v0::result::HandlerResult;
use warp::{Rejection, Reply};

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_tags(context: ContextLock) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();
    info!("get_tags");
    let entries = context.state().voters().tags();
    Ok(warp::reply::json(&entries))
}

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_voters_info(
    tag: String,
    voting_key: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();
    info!("get_voters_info");
    let last_updated = context
        .state()
        .voters()
        .snapshot_by_tag(&tag)
        .ok_or_else(|| HandleError::NotFound(format!("snapshot tag not found '{}'", tag)))?;
    let mut voter_info = Vec::new();
    let voters = context
        .state()
        .voters()
        .voters_by_voting_key_and_snapshot_tag(&voting_key, &tag);

    for voter in voters {
        let contributors = context
            .state()
            .voters()
            .contributions_by_voting_key_and_voter_group_and_snapshot_tag(
                &voting_key,
                &voter.voting_group,
                &tag,
            );

        let total_voting_power_per_group = context
            .state()
            .voters()
            .total_voting_power_by_voting_group_and_snapshot_tag(&voter.voting_group, &tag)
            as f64;

        voter_info.push(VoterInfo {
            voting_power: Value(voter.voting_power as u64).into(),
            delegations_count: contributors.len() as u64,
            delegations_power: contributors
                .iter()
                .map(|contributor| contributor.value as u64)
                .sum(),
            voting_group: voter.voting_group.clone(),
            voting_power_saturation: if total_voting_power_per_group != 0_f64 {
                voter.voting_power as f64 / total_voting_power_per_group
            } else {
                0_f64
            },
        })
    }

    Ok(HandlerResult(Ok(VotersInfo {
        voter_info,
        last_updated: *last_updated,
    })))
}

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_delegator_info(
    tag: String,
    stake_public_key: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();
    info!("get_delegator_info");
    let last_updated = context
        .state()
        .voters()
        .snapshot_by_tag(&tag)
        .ok_or_else(|| HandleError::NotFound(format!("snapshot tag not found '{}'", tag)))?;
    let contributions = context
        .state()
        .voters()
        .contributions_by_stake_public_key_and_snapshot_tag(&stake_public_key, &tag);

    Ok(HandlerResult(Ok(DelegatorInfo {
        dreps: contributions
            .iter()
            .map(|contribution| contribution.voting_key.clone())
            .unique()
            .collect(),
        voting_groups: contributions
            .iter()
            .map(|contribution| contribution.voting_group.clone())
            .unique()
            .collect(),
        last_updated: *last_updated,
    })))
}

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_challenges(context: ContextLock) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();

    info!("get_challenges");

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }
    Ok(HandlerResult(Ok(context.state().vit().challenges())))
}

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_challenge_by_id(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();

    info!("get_challenge_by_id {} ...", id);

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    let challenge = context
        .state()
        .vit()
        .challenges()
        .iter()
        .find(|&ch| ch.id == id)
        .cloned()
        .ok_or_else(|| HandleError::NotFound(id.to_string()))?;
    let proposals: Vec<Proposal> = context
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
#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_challenge_by_id_and_group_id(
    id: i32,
    group_id: impl Into<String> + Debug,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();
    let group_id = group_id.into();

    info!("get_challenge_by_id {} and group id {} ...", id, group_id);

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    let challenge = context
        .state()
        .vit()
        .challenges()
        .iter()
        .find(|&ch| ch.id == id)
        .cloned()
        .ok_or_else(|| HandleError::NotFound(id.to_string()))?;
    let proposals: Vec<Proposal> = context
        .state()
        .vit()
        .proposals()
        .iter()
        .filter(|x| x.proposal.challenge_id == challenge.id && x.group_id == group_id)
        .map(|x| x.proposal.clone())
        .collect();

    if proposals.is_empty() {
        return Err(warp::reject::custom(NoChallengeIdAndGroupFound {
            id: id.to_string(),
            group: group_id,
        }));
    }

    Ok(HandlerResult(Ok(ChallengeWithProposals {
        challenge,
        proposals,
    })))
}

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_review_by_id(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();

    info!("get_review_by_id {} ...", id);

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    let reviews: HashMap<String, _> = context
        .state()
        .vit()
        .advisor_reviews()
        .iter()
        .filter(|&review| review.proposal_id == id)
        .cloned()
        .group_by(|review| review.assessor.to_string())
        .into_iter()
        .map(|(key, group)| (key, group.collect::<Vec<_>>()))
        .collect();

    Ok(HandlerResult(Ok(reviews)))
}

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_all_proposals(
    voting_group: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();

    info!("get_all_proposals");

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    Ok(HandlerResult(Ok(context
        .state()
        .vit()
        .proposals()
        .into_iter()
        .filter(|p| p.group_id == voting_group)
        .collect::<Vec<_>>())))
}

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_proposal_by_idx(
    request: ProposalsByVoteplanIdAndIndex,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();

    info!("get_proposal_by_idx ({:?}) ...", request);

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    let proposals: Vec<_> = context
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

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_proposal(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();

    info!("get_proposal {} ...", id);

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    let proposal = context
        .state()
        .vit()
        .proposals()
        .iter()
        .find(|x| x.proposal.internal_id.to_string() == id.to_string())
        .cloned()
        .ok_or_else(|| warp::reject::custom(GeneralException::proposal_not_found(id)))?;

    Ok(HandlerResult(Ok(proposal)))
}

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_fund_by_id(id: i32, context: ContextLock) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();

    info!("get_fund_by_id {} ...", id);

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    let funds = context.state().vit().funds();
    let fund = funds.iter().find(|x| x.id == id).unwrap();

    Ok(HandlerResult(Ok(fund.clone())))
}

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_fund(context: ContextLock) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();

    info!("get_fund");

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    let funds: Vec<Fund> = context.state().vit().funds().to_vec();
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

#[tracing::instrument(skip(context), name = "REST Api call")]
pub async fn get_all_funds(context: ContextLock) -> Result<impl Reply, Rejection> {
    let context = context.read().unwrap();

    info!("get_all_funds");

    if let Some(error_code) = context.check_if_rest_available() {
        return Err(warp::reject::custom(error_code));
    }

    let funds: Vec<Fund> = context.state().vit().funds().to_vec();

    Ok(warp::reply::json(&funds))
}
