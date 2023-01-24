use std::collections::HashMap;

use bigdecimal::BigDecimal;
use color_eyre::{Report, Result};
use diesel::{BoolExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl};

use crate::data::StakeKeyHex;
use crate::db::inner::DbQuery;
use crate::db::Db;
use diesel::sql_types::Text;
use diesel::ExpressionMethods;

sql_function! (fn decode(string: Text, format: Text) -> Bytea);

impl Db {
    /// Query the stake values
    ///
    /// This query is detailed in <../design/stake_value_processing.md>
    #[instrument]
    pub fn stake_values(
        &self,
        stake_addrs: &[StakeKeyHex],
    ) -> Result<HashMap<StakeKeyHex, BigDecimal>> {
        let rows = stake_addrs.iter().map(|addr| {
            let hex = hex::encode(addr.0);
            let result = self.exec(|conn| query(hex).load(conn))?;
            // this clone is actually a copy, it's only needed because the underlying Ed25519 type
            // doesn't implement `Copy` even though it's just a byte array
            Ok::<_, Report>((addr.clone(), result))
        });

        // If performance becomes an issue, we can replace this with `dashmap` and parallelize the
        // loop with `rayon`
        let mut result = HashMap::with_capacity(rows.len());

        for row in rows {
            let (addr, values) = row?;
            let sum: BigDecimal = values.iter().sum();
            *result.entry(addr).or_insert_with(|| BigDecimal::from(0)) += sum;
        }

        Ok(result)
    }
}

fn query(stake_addr: String) -> impl DbQuery<'static, BigDecimal> {
    use crate::db::schema::{stake_address, tx_in, tx_out};

    let outer_join = tx_in::table.on(tx_out::tx_id
        .eq(tx_in::tx_out_id)
        .and(tx_out::index.eq(tx_in::tx_out_index)));

    let inner_join =
        stake_address::table.on(stake_address::id.nullable().eq(tx_out::stake_address_id));

    let table = tx_out::table
        .left_outer_join(outer_join)
        .inner_join(inner_join)
        .filter(tx_in::tx_in_id.is_null());

    table
        .select(tx_out::value)
        .filter(stake_address::hash_raw.eq(decode(stake_addr, "hex")))
}
