use std::collections::{HashMap, HashSet};

use chain_impl_mockchain::value::Value;
use color_eyre::{
    eyre::{bail, eyre},
    Result,
};
use itertools::Itertools;
use jormungandr_lib::{
    crypto::hash::Hash,
    interfaces::{
        Address, Block0Configuration, Initial, Tally, VotePlanStatus, VoteProposalStatus,
    },
};
use log::debug;
use vit_servicing_station_lib::db::models::{challenges::Challenge, proposals::Proposal};

pub use types::*;
pub use util::build_path_for_challenge;

use self::{io::vecs_to_maps, types::NotFundedReason};

pub mod io;
mod types;
mod util;

pub struct ProposerRewardsInputs {
    pub block0_config: Block0Configuration,
    pub proposals: Vec<Proposal>,
    pub voteplans: Vec<VotePlanStatus>,
    pub challenges: Vec<Challenge>,
    pub excluded_proposals: HashSet<String>,
    pub committee_keys: Vec<Address>,
    pub total_stake_threshold: f64,
    pub approval_threshold: f64,
}

pub fn proposer_rewards(
    ProposerRewardsInputs {
        block0_config,
        proposals,
        voteplans,
        challenges,
        excluded_proposals,
        committee_keys,
        total_stake_threshold,
        approval_threshold,
    }: ProposerRewardsInputs,
) -> Result<Vec<(Challenge, Vec<Calculation>)>> {
    let (proposals, voteplans, challenges) = vecs_to_maps(proposals, voteplans, challenges)?;
    sanity_check_data(&proposals, &voteplans)?;

    let proposals = filter_excluded_proposals(&proposals, &excluded_proposals);

    let Value(total_stake) = calculate_total_stake_from_block0(&block0_config, &committee_keys);
    let total_stake_approval_threshold = total_stake_threshold * total_stake as f64;

    let mut result = Vec::with_capacity(challenges.len());

    for (id, challenge) in challenges {
        let (challenge_proposals, challenge_voteplan_proposals) =
            filter_data_by_challenge(id, &proposals, &voteplans);

        let calculations = calculate_results(
            &challenge_proposals,
            &challenge_voteplan_proposals,
            challenge.rewards_total,
            approval_threshold,
            total_stake_approval_threshold,
        )?;

        result.push((challenge, calculations));
    }

    Ok(result)
}

pub fn calculate_results(
    proposals: &HashMap<Hash, Proposal>,
    voteplans: &HashMap<Hash, VoteProposalStatus>,
    fund: i64,
    threshold: f64,
    total_stake_threshold: f64,
) -> Result<Vec<Calculation>> {
    debug!("calculating. threshold: {threshold}, total_stake_threshold: {total_stake_threshold}");
    let success_results = calculate_vote_difference_and_threshold_success(
        proposals,
        voteplans,
        threshold,
        total_stake_threshold,
    )?;

    let mut sorted_ids = success_results.keys().collect_vec();
    sorted_ids.sort_unstable_by_key(|&id| success_results[id].0);

    let mut results = vec![];
    let mut depletion = fund;

    for proposal_id in sorted_ids {
        debug!("calculating proposal_id: {proposal_id}");
        let proposal = &proposals[proposal_id];
        let voteplan = &voteplans[proposal_id];
        let (total_result, threshold_success) = success_results[proposal_id];
        let (yes, no) = extract_yes_no_votes(proposal, voteplan)?;

        let funded = threshold_success && depletion > 0 && depletion >= proposal.proposal_funds;

        let not_funded_reason = match (funded, threshold_success) {
            (true, _) => None,
            (false, true) => Some(NotFundedReason::OverBudget),
            (false, false) => Some(NotFundedReason::ApprovalThreshold),
        };
        debug!("not funded reason: {not_funded_reason:?}");

        if funded {
            depletion -= proposal.proposal_funds;
        }

        results.push(Calculation {
            internal_id: proposal.proposal_id.clone(),
            proposal_id: *proposal_id,
            proposal: proposal.proposal_title.clone(),
            overall_score: proposal.proposal_impact_score / 100,
            yes,
            no,
            result: total_result,
            meets_approval_threshold: threshold_success.into(),
            requested_dollars: proposal.proposal_funds,
            status: funded.into(),
            fund_depletion: depletion as f64,
            not_funded_reason,
            link_to_ideascale: proposal.proposal_url.clone(),
        });
    }

    Ok(results)
}

fn calculate_vote_difference_and_threshold_success(
    proposals: &HashMap<Hash, Proposal>,
    voteplans: &HashMap<Hash, VoteProposalStatus>,
    threshold: f64,
    total_stake_threshold: f64,
) -> Result<HashMap<Hash, (i64, bool)>> {
    let result = proposals
        .iter()
        .map(|(id, prop)| {
            let voteplan = voteplans
                .get(id)
                .ok_or(eyre!("no voteplan with id: {id}"))?;
            let answer =
                calculate_approval_threshold(prop, voteplan, threshold, total_stake_threshold)?;

            Ok((*id, answer))
        })
        .collect::<Result<_>>()?;

    Ok(result)
}

fn calculate_approval_threshold(
    proposal: &Proposal,
    voteplan: &VoteProposalStatus,
    threshold: f64,
    total_stake_threshold: f64,
) -> Result<(i64, bool)> {
    debug!(
        "calculating approval threshold for proposal_id: {}",
        &proposal.proposal_id
    );

    let (yes, no) = extract_yes_no_votes(proposal, voteplan)?;
    debug!("yes votes: {yes}, no votes: {no}");

    let total = yes + no;
    let diff = yes as i64 - no as i64;
    debug!("total: {total}, diff: {diff}");

    let pass_total_threshold = total as f64 >= total_stake_threshold;
    let pass_relative_threshold = (yes as f64 / no as f64) >= threshold;
    let success = pass_total_threshold && pass_relative_threshold;

    debug!("total_stake_threshold: {total_stake_threshold}, threshold: {threshold}");
    debug!("success: {success}, total_threshold: {pass_total_threshold}, relative_threshold: {pass_relative_threshold}");

    Ok((diff, success))
}

/// returns (yes, no)
fn extract_yes_no_votes(proposal: &Proposal, voteplan: &VoteProposalStatus) -> Result<(u64, u64)> {
    let yes_index = proposal
        .chain_vote_options
        .0
        .get("yes")
        .ok_or(eyre!("missing `yes` field"))?;
    let no_index = proposal
        .chain_vote_options
        .0
        .get("no")
        .ok_or(eyre!("missing `no` field"))?;

    let tally = match &voteplan.tally {
        Tally::Public { result } => result,
        Tally::Private { .. } => bail!("private vote tally"),
    };

    let yes_result = tally.results()[*yes_index as usize];
    let no_result = tally.results()[*no_index as usize];

    Ok((yes_result, no_result))
}

fn filter_data_by_challenge(
    challenge_id: i32,
    proposals: &HashMap<Hash, Proposal>,
    voteplans: &HashMap<Hash, VoteProposalStatus>,
) -> (HashMap<Hash, Proposal>, HashMap<Hash, VoteProposalStatus>) {
    let proposals: HashMap<_, _> = proposals
        .iter()
        .filter(|(_, prop)| prop.challenge_id == challenge_id)
        .map(|(k, v)| (*k, v.clone()))
        .collect();

    let voteplans = voteplans
        .iter()
        .filter(|(_, plan)| proposals.contains_key(&plan.proposal_id))
        .map(|(k, v)| (*k, v.clone()))
        .collect();

    (proposals, voteplans)
}

fn sanity_check_data(
    proposals: &HashMap<Hash, Proposal>,
    voteplans: &HashMap<Hash, VoteProposalStatus>,
) -> Result<()> {
    let proposals_set: HashSet<_> = proposals.keys().copied().collect();
    let voteplan_proposals_set: HashSet<_> = voteplans.keys().copied().collect();

    if proposals_set != voteplan_proposals_set {
        let diff = proposals_set
            .symmetric_difference(&voteplan_proposals_set)
            .join(", ");
        bail!("proposal id set inconsistency. Bad ids: {}", diff);
    }

    Ok(())
}

fn filter_excluded_proposals(
    proposals: &HashMap<Hash, Proposal>,
    excluded_proposals: &HashSet<String>,
) -> HashMap<Hash, Proposal> {
    let predicate = |prop: &Proposal| {
        let chain_proposal_id = String::from_utf8(prop.chain_proposal_id.clone()).unwrap();

        !excluded_proposals.contains(&prop.proposal_id)
            && !excluded_proposals.contains(&chain_proposal_id)
    };

    proposals
        .iter()
        .filter(|(_, prop)| predicate(prop))
        .map(|(id, prop)| (*id, prop.clone()))
        .collect()
}

fn calculate_total_stake_from_block0(
    block0_config: &Block0Configuration,
    committee_keys: &[Address],
) -> Value {
    block0_config
        .initial
        .iter()
        .filter_map(|initial| match initial {
            Initial::Fund(fund) => Some(fund),
            _ => None,
        })
        .flatten()
        .filter_map(|initial| {
            if committee_keys.contains(&initial.address) {
                None
            } else {
                Some(Value::from(initial.value))
            }
        })
        .sum()
}
