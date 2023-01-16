use bigdecimal::BigDecimal;
use diesel::sql_types::{Numeric, Timestamp};
use diesel::{Queryable, QueryableByName};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Queryable, PartialEq, Eq, Debug, Serialize, Clone)]
pub struct Meta {
    pub id: i64,
    pub start_time: SystemTime,
    pub version: String,
    pub network_name: String,
}

#[derive(QueryableByName, Debug, Serialize, Deserialize, Clone)]
pub struct BehindDuration {
    #[diesel(sql_type = Timestamp)]
    pub behind_by: SystemTime,
}

#[derive(QueryableByName, Debug, Serialize, Deserialize, Clone)]
pub struct Progress {
    #[diesel(sql_type = Numeric)]
    pub sync_percentage: BigDecimal,
}

#[derive(Debug, Serialize)]
pub struct TransactionConfirmation {
    pub epoch_no: Option<i32>,
    pub slot_no: Option<i64>,
    pub absolute_slot: Option<i32>,
    pub block_no: Option<i32>,
}

impl From<TransactionConfirmationRow> for TransactionConfirmation {
    fn from(row: TransactionConfirmationRow) -> Self {
        TransactionConfirmation {
            epoch_no: row.0,
            slot_no: row.1,
            absolute_slot: row.2,
            block_no: row.3,
        }
    }
}

pub type TransactionConfirmationRow = (Option<i32>, Option<i64>, Option<i32>, Option<i32>);
