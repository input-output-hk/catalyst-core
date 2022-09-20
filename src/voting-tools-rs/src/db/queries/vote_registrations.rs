use crate::{db::inner::DbQuery, Db};

use crate::db::schema::{block, tx, tx_metadata};
use crate::model::{Reg, SlotNo};
use bigdecimal::{BigDecimal, FromPrimitive};
use color_eyre::eyre::Result;
use diesel::RunQueryDsl;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgAnyJsonExpressionMethods,
    PgJsonbExpressionMethods, QueryDsl,
};
use once_cell::sync::Lazy;
use serde_json::Value;

static METADATA_KEY: Lazy<BigDecimal> = Lazy::new(|| BigDecimal::from_isize(61284).unwrap());
static SIGNATURE_KEY: Lazy<BigDecimal> = Lazy::new(|| BigDecimal::from_isize(61285).unwrap());

type Row = (i64, Option<Value>, Option<Value>, Option<i64>);

impl Db {
    /// This query is detailed in ``design/vote_registration_query.md``
    ///
    /// 'lower' is an optional inclusive lower bound. If `None`, `0` is used instead.
    /// 'upper' is an optional inclusive upper bound. If `None`, `i64::MAX` is used instead.
    ///
    /// Returns an error if either `lower` or `upper` is greater than `i64::MAX`
    ///
    /// # Errors
    ///
    /// Returns an error if either of `lower` or `upper` doesn't fit in an `i64`
    pub fn vote_registrations(
        &self,
        lower: Option<SlotNo>,
        upper: Option<SlotNo>,
    ) -> Result<Vec<Reg>> {
        let lower = lower.unwrap_or(SlotNo(0)).into_i64()?;
        let upper = upper.unwrap_or(SlotNo(i64::MAX as u64)).into_i64()?;
        let q = query(lower, upper);

        let results = self.exec(move |conn| q.load(conn))?;
        let results = process(results);

        Ok(results)
    }
}

/// This query doesn't exactly match the query in the doc. In particular, some filtering of invalid
/// JSONs isn't performed, since it breaks something (not sure what, just get no rows back). For
/// now, we'll just do the filtering in Rust
fn query(lower: i64, upper: i64) -> impl DbQuery<'static, Row> {
    let (meta_table, sig_table) = alias!(tx_metadata as meta_table, tx_metadata as sig_table);

    let metadata = meta_table.field(tx_metadata::json);
    let signature = sig_table.field(tx_metadata::json);

    let tables = meta_table
        .inner_join(tx::table.on(tx::id.eq(meta_table.field(tx_metadata::tx_id))))
        .inner_join(
            sig_table.on(sig_table
                .field(tx_metadata::tx_id)
                .eq(meta_table.field(tx_metadata::tx_id))),
        )
        .inner_join(block::table.on(block::id.eq(tx::block_id)));

    let selection = (
        meta_table.field(tx_metadata::tx_id),
        metadata,
        signature,
        block::slot_no,
    );

    let signature_keys_predicate = signature.has_all_keys(["1"].as_slice());
    let block_number_predicate = block::slot_no.ge(lower).and(block::slot_no.le(upper));

    let metadata_2 = metadata.retrieve_as_object("2");

    tables
        .filter(meta_table.field(tx_metadata::key).eq(&*METADATA_KEY))
        .filter(sig_table.field(tx_metadata::key).eq(&*SIGNATURE_KEY))
        .filter(signature_keys_predicate)
        .filter(block_number_predicate)
        .select(selection)
        .distinct_on(metadata_2)
}

fn process(rows: Vec<Row>) -> Vec<Reg> {
    rows.into_iter()
        .filter_map(|(tx_id, metadata, signature, _)| {
            let tx_id = u64::try_from(tx_id).ok()?.into();
            let metadata = serde_json::from_value(metadata?).ok()?;
            let signature = serde_json::from_value(signature?).ok()?;

            Some(Reg {
                tx_id,
                metadata,
                signature,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value, json};

    use super::*;

    fn good_meta() -> serde_json::Value {
        json!({
            "1": "legacy",
            "2": "stakevkey",
            "3": "rewardsaddr",
            "4": 123,
        })
    }

    fn good_sig() -> serde_json::Value {
        json!({
            "1": "sig",
        })
    }

    #[test]
    fn process_happy_path() {
        let rows = vec![(1, Some(good_meta()), Some(good_sig()), None)];
        let regs = vec![Reg {
            tx_id: 1.into(),
            metadata: from_value(good_meta()).unwrap(),
            signature: from_value(good_sig()).unwrap(),
        }];

        assert_eq!(process(rows), regs);
    }

    #[test]
    fn filters_bad_rows() {
        fn check(row: Row) {
            assert!(process(vec![row]).is_empty());
        }

        // bad sig
        check((1, Some(good_meta()), Some(json!("random json")), None));

        // bad meta
        check((1, Some(json!("random json")), Some(good_sig()), None));

        // none
        check((1, None, None, None));
    }
}
