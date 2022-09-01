use bigdecimal::BigDecimal;
use color_eyre::Result;
use diesel::{BoolExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl};

use crate::{db::Db, model::SlotNo};
use diesel::sql_types::{Bytea, Text};

sql_function! (fn decode(string: Text, format: Text) -> Bytea);

impl Db {
    #[instrument]
    pub fn stake_value(&self, stake_address_hex: &str) -> Result<BigDecimal> {
        let rows: Vec<BigDecimal> = self.exec(move |conn| {
            use crate::db::schema::*;
            use diesel::ExpressionMethods;

            let outer_join = tx_in::table.on(tx_out::tx_id
                .eq(tx_in::tx_out_id)
                .and(tx_out::index.eq(tx_in::tx_out_index)));

            let inner_join =
                stake_address::table.on(stake_address::id.nullable().eq(tx_out::stake_address_id));

            let join = tx_out::table
                .left_outer_join(outer_join)
                .inner_join(inner_join)
                .filter(tx_in::tx_in_id.is_null());

            join.select(tx_out::value)
                .filter(stake_address::hash_raw.eq(decode(stake_address_hex, "hex")))
                .load(conn)
        })?;

        Ok(rows.iter().sum())
        // let conn = self.conn().await?;
        // let rows = conn.query("SELECT utxo_snapshot.value FROM utxo_snapshot WHERE stake_credential = decode('$1', 'hex');", &[&stake_address_hex]).await?;
        //
        // let sum = rows
        //     .into_iter()
        //     .map(|row| {
        //         let value: Decimal = row.get(0);
        //         u64::try_from(value).unwrap_or(0)
        //     })
        //     .sum();
        //
        // Ok(sum)
    }

    #[instrument]
    pub fn stake_values(
        &self,
        slot_no: Option<SlotNo>,
        stake_addrs: &[String],
    ) -> Result<Vec<(String, BigDecimal)>> {
        stake_addrs
            .iter()
            .map(|s| -> Result<(String, BigDecimal)> { Ok((s.to_string(), self.stake_value(&s)?)) })
            .collect()
    }
}
