#![allow(clippy::extra_unused_lifetimes)]

use crate::db::schema::{contributions, snapshots, voters};
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = "snapshots")]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    /// Tag - a unique identifier of the current snapshot
    pub tag: String,
    /// Timestamp for the latest update of the current snapshot
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    pub last_updated: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = "voters")]
#[serde(rename_all = "camelCase")]
pub struct Voter {
    pub voting_key: String,
    pub voting_power: i64,
    pub voting_group: String,
    pub snapshot_tag: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = "contributions")]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub stake_public_key: String,
    pub reward_address: String,
    pub value: i64,
    pub voting_key: String,
    pub voting_group: String,
    pub snapshot_tag: String,
}
