use crate::{db::inner::DbQuery, Db};

use crate::db::schema::*;
use crate::model::SlotNo;
use bigdecimal::{BigDecimal, FromPrimitive};
use color_eyre::eyre::Result;
use diesel::{
    pg::Pg, select, BoolExpressionMethods, ExpressionMethods, JoinOnDsl,
    PgAnyJsonExpressionMethods, PgJsonbExpressionMethods, QueryDsl,
};
use once_cell::sync::Lazy;
use serde_json::Value;

static METADATA_KEY: Lazy<BigDecimal> = Lazy::new(|| BigDecimal::from_isize(61284).unwrap());
static SIGNATURE_KEY: Lazy<BigDecimal> = Lazy::new(|| BigDecimal::from_isize(61285).unwrap());

impl Db {
    /// This query is detailed in <../design/vote_registration_query.md>
    ///
    /// 'lower' is an optional inclusive lower bound. If `None`, `0` is used instead.
    /// 'upper' is an optional inclusive upper bound. If `None`, `i64::MAX` is used instead.
    ///
    /// Returns an error if either `lower` or `upper` is greater than `i64::MAX`
    pub async fn opt_vote_registration(lower: Option<SlotNo>, upper: Option<SlotNo>) -> Result<()> {
        let lower = lower.unwrap_or(SlotNo(0)).into_i64()?;
        let upper = upper.unwrap_or(SlotNo(i64::MAX as u64)).into_i64()?;
        todo!()
    }
}

fn metadata_query(lower: i64, upper: i64) -> impl DbQuery<'static, (i64, Option<Value>)> {
    use tx_metadata::*;
    let slot_no_predicate = block::slot_no.ge(lower).and(block::slot_no.le(upper));
    tx_metadata::table
        .inner_join(tx::table.on(tx::id.eq(tx_metadata::tx_id)))
        .inner_join(block::table.on(block::id.eq(tx::block_id)))
        .filter(tx_metadata::key.eq(&*METADATA_KEY))
        .filter(slot_no_predicate)
        .select((tx_id, json))
        .distinct_on(tx_metadata::json.retrieve_as_object("2")) // postgres `->` operator
}

// fn signature_query(lower: i64, upper: i64) -> impl DbQuery<'static,
