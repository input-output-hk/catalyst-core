use csv::Writer;
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};
use vit_servicing_station_lib::{
    db::{
        self, load_db_connection_pool,
        queries::{
            funds::query_fund_by_id,
            snapshot::{
                query_contributions_by_snapshot_tag, query_snapshot_by_tag,
                query_voters_by_snapshot_tag,
            },
        },
    },
    utils::serde::{deserialize_unix_timestamp_from_rfc3339, serialize_unix_timestamp_as_rfc3339},
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Fund {
    #[serde(default = "Default::default")]
    pub id: i32,
    #[serde(alias = "fundName")]
    pub fund_name: String,
    #[serde(alias = "fundGoal")]
    pub fund_goal: String,
    #[serde(alias = "votingPowerThreshold")]
    pub voting_power_threshold: i64,
    #[serde(alias = "fundStartTime")]
    #[serde(deserialize_with = "deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    pub fund_start_time: i64,
    #[serde(alias = "fundEndTime")]
    #[serde(deserialize_with = "deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    pub fund_end_time: i64,
    #[serde(alias = "nextFundStartTime")]
    #[serde(deserialize_with = "deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    pub next_fund_start_time: i64,
    #[serde(alias = "registrationSnapshotTime")]
    #[serde(deserialize_with = "deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    pub registration_snapshot_time: i64,
    #[serde(alias = "nextRegistrationSnapshotTime")]
    #[serde(deserialize_with = "deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "serialize_unix_timestamp_as_rfc3339")]
    pub next_registration_snapshot_time: i64,
    // #[serde(flatten)]
    // pub stage_dates: FundStageDates,
    #[serde(alias = "resultsUrl")]
    pub results_url: String,
    #[serde(alias = "surveyUrl")]
    pub survey_url: String,
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
        // stage_dates: fund.stage_dates,
        results_url: fund.results_url,
        survey_url: fund.survey_url,
    })?;
    chanllenges_writer.serialize(fund.challenges)?;
    vote_plans_writer.serialize(fund.chain_vote_plans)?;
    goals_writer.serialize(fund.goals)?;
    groups_writer.serialize(fund.groups)?;

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

    snapshot_writer.serialize(snapshot)?;
    voters_writer.serialize(voters)?;
    contributions_writer.serialize(contributions)?;

    Ok(())
}
