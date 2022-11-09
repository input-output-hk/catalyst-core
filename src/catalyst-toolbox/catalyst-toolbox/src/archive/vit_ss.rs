use csv::Writer;
use serde::Serialize;
use std::{fs::File, path::Path};
use vit_servicing_station_lib::{
    db::{
        self, load_db_connection_pool,
        queries::{
            funds::query_fund_by_id,
            proposals::query_all_proposals,
            snapshot::{
                query_contributions_by_snapshot_tag, query_snapshot_by_tag,
                query_voters_by_snapshot_tag,
            },
        },
    },
    utils::serde::{serialize_bin_as_str, serialize_unix_timestamp_as_rfc3339},
    v0::errors::HandleError,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DatabaseError(#[from] db::Error),

    #[error(transparent)]
    FetchError(#[from] HandleError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Csv(#[from] csv::Error),
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Fund {
    id: i32,
    fund_name: String,
    fund_goal: String,
    voting_power_threshold: i64,
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    fund_start_time: i64,
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    fund_end_time: i64,
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    next_fund_start_time: i64,
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    registration_snapshot_time: i64,
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    next_registration_snapshot_time: i64,
    results_url: String,
    survey_url: String,
}

#[derive(Serialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Proposal {
    internal_id: i32,
    proposal_id: String,
    category_id: String,
    category_name: String,
    category_description: String,
    proposal_title: String,
    proposal_summary: String,
    proposal_public_key: String,
    proposal_funds: i64,
    proposal_url: String,
    proposal_files_url: String,
    proposal_impact_score: i64,
    proposer_name: String,
    proposer_email: String,
    proposer_url: String,
    proposer_relevant_experience: String,
    #[serde(serialize_with = "serialize_bin_as_str")]
    chain_proposal_id: Vec<u8>,
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    chain_vote_start_time: i64,
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    chain_vote_end_time: i64,
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    chain_committee_end_time: i64,
    chain_voteplan_payload: String,
    chain_vote_encryption_key: String,
    fund_id: i32,
    challenge_id: i32,
    reviews_count: i32,
}

fn csv_writer(output_dir: &Path, name: String) -> Result<Writer<File>, Error> {
    let mut path = output_dir.to_path_buf();
    path.push(name);
    let file = std::fs::File::create(path).unwrap();
    Ok(csv::Writer::from_writer(file))
}

pub async fn generate_archive_files(
    vit_ss_database: &str,
    output_dir: &Path,
    fund_id: i32,
    snapshot_tag: String,
) -> Result<(), Error> {
    let db_pool = load_db_connection_pool(vit_ss_database)?;

    // fund info
    let mut fund_writer = csv_writer(output_dir, format!("fund_{}.csv", fund_id))?;
    let mut chanllenges_writer = csv_writer(output_dir, format!("challenges_{}.csv", fund_id))?;
    let mut vote_plans_writer = csv_writer(output_dir, format!("vote_plans_{}.csv", fund_id))?;
    let mut goals_writer = csv_writer(output_dir, format!("goals_{}.csv", fund_id))?;
    let mut groups_writer = csv_writer(output_dir, format!("groups_{}.csv", fund_id))?;

    let fund = query_fund_by_id(fund_id, &db_pool).await?;

    fund_writer.serialize(Fund {
        id: fund.id,
        fund_name: fund.fund_name,
        fund_goal: fund.fund_goal,
        voting_power_threshold: fund.voting_power_threshold,
        fund_start_time: fund.fund_start_time,
        fund_end_time: fund.fund_end_time,
        next_fund_start_time: fund.next_fund_start_time,
        registration_snapshot_time: fund.registration_snapshot_time,
        next_registration_snapshot_time: fund.next_registration_snapshot_time,
        results_url: fund.results_url,
        survey_url: fund.survey_url,
    })?;
    chanllenges_writer.serialize(&fund.challenges)?;
    vote_plans_writer.serialize(&fund.chain_vote_plans)?;
    goals_writer.serialize(&fund.goals)?;
    groups_writer.serialize(&fund.groups)?;

    // snapshot info
    let mut snapshot_writer =
        csv_writer(output_dir, format!("snapshot_{}.csv", snapshot_tag.clone()))?;
    let mut voters_writer = csv_writer(output_dir, format!("voters_{}.csv", snapshot_tag.clone()))?;
    let mut contributions_writer = csv_writer(
        output_dir,
        format!("contributions_{}.csv", snapshot_tag.clone()),
    )?;

    let snapshot = query_snapshot_by_tag(snapshot_tag.clone(), &db_pool).await?;
    let voters = query_voters_by_snapshot_tag(snapshot_tag.clone(), &db_pool).await?;
    let contributions = query_contributions_by_snapshot_tag(snapshot_tag, &db_pool).await?;

    snapshot_writer.serialize(&snapshot)?;
    voters_writer.serialize(&voters)?;
    contributions_writer.serialize(&contributions)?;

    // proposals info
    for voting_group in fund.groups {
        let mut proposals_writer = csv_writer(
            output_dir,
            format!(
                "proposals_{}_{}.csv",
                voting_group.group_id.clone(),
                fund_id
            ),
        )?;

        let proposals = query_all_proposals(&db_pool, voting_group.group_id).await?;

        proposals_writer.serialize(
            proposals
                .into_iter()
                .map(|el| Proposal {
                    internal_id: el.proposal.internal_id,
                    proposal_id: el.proposal.proposal_id,
                    category_id: el.proposal.proposal_category.category_id,
                    category_name: el.proposal.proposal_category.category_name,
                    category_description: el.proposal.proposal_category.category_description,
                    proposal_title: el.proposal.proposal_title,
                    proposal_summary: el.proposal.proposal_summary,
                    proposal_public_key: el.proposal.proposal_public_key,
                    proposal_funds: el.proposal.proposal_funds,
                    proposal_url: el.proposal.proposal_url,
                    proposal_files_url: el.proposal.proposal_files_url,
                    proposal_impact_score: el.proposal.proposal_impact_score,
                    proposer_name: el.proposal.proposer.proposer_name,
                    proposer_email: el.proposal.proposer.proposer_email,
                    proposer_url: el.proposal.proposer.proposer_url,
                    proposer_relevant_experience: el.proposal.proposer.proposer_relevant_experience,
                    chain_proposal_id: el.proposal.chain_proposal_id,
                    chain_vote_start_time: el.proposal.chain_vote_start_time,
                    chain_vote_end_time: el.proposal.chain_vote_end_time,
                    chain_committee_end_time: el.proposal.chain_committee_end_time,
                    chain_voteplan_payload: el.proposal.chain_voteplan_payload,
                    chain_vote_encryption_key: el.proposal.chain_vote_encryption_key,
                    fund_id: el.proposal.fund_id,
                    challenge_id: el.proposal.challenge_id,
                    reviews_count: el.proposal.reviews_count,
                })
                .collect::<Vec<Proposal>>(),
        )?;
    }

    Ok(())
}
