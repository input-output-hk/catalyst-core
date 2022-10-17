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

// impl Insertable<snapshots::table> for Snapshot {
//     type Values = (
//         diesel::dsl::Eq<snapshots::tag, String>,
//         diesel::dsl::Eq<snapshots::last_updated, i64>,
//     );

//     fn values(self) -> Self::Values {
//         (
//             snapshots::tag.eq(self.tag),
//             snapshots::last_updated.eq(self.last_updated),
//         )
//     }
// }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = "voters")]
#[serde(rename_all = "camelCase")]
pub struct Voter {
    pub voting_key: String,
    pub voting_power: i64,
    pub voting_group: String,
    pub snapshot_tag: String,
}

// impl Insertable<voters::table> for Voter {
//     type Values = (
//         diesel::dsl::Eq<voters::voting_key, String>,
//         diesel::dsl::Eq<voters::voting_power, i64>,
//         diesel::dsl::Eq<voters::voting_group, String>,
//         diesel::dsl::Eq<voters::snapshot_tag, String>,
//     );

//     fn values(self) -> Self::Values {
//         (
//             voters::voting_key.eq(self.voting_key),
//             voters::voting_power.eq(self.voting_power),
//             voters::voting_group.eq(self.voting_group),
//             voters::snapshot_tag.eq(self.snapshot_tag),
//         )
//     }
// }

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
