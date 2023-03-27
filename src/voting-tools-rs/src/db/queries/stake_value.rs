#![allow(missing_docs)]

use std::sync::atomic::{AtomicU64, Ordering};

use bigdecimal::BigDecimal;
use color_eyre::Report;
use dashmap::DashMap;
use diesel::{BoolExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl};

use crate::data::StakeKeyHex;
use crate::db::inner::DbQuery;
use crate::db::Db;
use crate::verify::StakeKeyHash;
use diesel::sql_types::Text;
use diesel::ExpressionMethods;

use rayon::prelude::*;

sql_function! (fn decode(string: Text, format: Text) -> Bytea);

impl Db {
    /// Query the stake values
    ///
    /// This query is detailed in <../design/stake_value_processing.md>
    ///
    /// # Errors
    ///
    /// Will return an error not at all.  TODO, don't return Result.
    pub fn stake_values(&self, stake_addrs: &[StakeKeyHash]) -> DashMap<StakeKeyHash, BigDecimal> {
        let rows = stake_addrs.par_iter().map(|addr| {
            let hex = hex::encode(&addr);
            let result = self.exec(|conn| query(hex).load(conn))?;
            Ok::<_, Report>((addr.clone(), result))
        });

        let max_rows = rows.len();

        // If performance becomes an issue, we can replace this with `dashmap` and parallelize the
        // loop with `rayon`
        // Sadly this is still chronically slow.
        // The DB code probably needs re-writing to use tokio_postgres and exploit its pipelining capabilities.
        // A 64 Thread machine is only getting 100 results every 4 seconds.
        // We need at least 86391 results from main net.
        // Total time to fetch = ~57 minutes.
        // For comparison the haskell voting tool does it in
        // This will slow down proportional to the number of cores, because rayon is not good for IO bound tasks.
        let result = DashMap::with_capacity(max_rows);

        let row_count = AtomicU64::new(0);

        rows.into_par_iter().for_each(|row| match row {
            Ok(row) => {
                let current_row = row_count.fetch_add(1, Ordering::SeqCst).saturating_add(1);
                let row_pct = current_row
                    .saturating_mul(100)
                    .saturating_div(max_rows as u64);

                if current_row % 100 == 0 {
                    info!("Stake Keys Processed: {current_row}/{max_rows} ({row_pct}%)");
                }
                let (addr, values) = row;
                let sum: BigDecimal = values.iter().sum();
                *result.entry(addr).or_insert_with(|| BigDecimal::from(0)) += sum;
            }
            Err(row) => {
                let current_row = row_count.fetch_add(1, Ordering::SeqCst).saturating_add(1);
                let row_pct = current_row
                    .saturating_mul(100)
                    .saturating_div(max_rows as u64);
                error!("Error on row: {current_row}/{max_rows} ({row_pct}%) :- {row}");
            }
        });

        let current_row = row_count.into_inner();
        info!("Stake Keys Processed: {current_row}/{max_rows} (100%)");

        result
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

    let result = table
        .select(tx_out::value)
        .filter(stake_address::hash_raw.eq(decode(stake_addr, "hex")));

    result
}
