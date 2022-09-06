use bigdecimal::BigDecimal;
use color_eyre::Result;
use diesel::{BoolExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl};

use crate::db::inner::DbQuery;
use crate::{db::Db, model::SlotNo};
use diesel::sql_types::Text;
use diesel::ExpressionMethods;

sql_function! (fn decode(string: Text, format: Text) -> Bytea);

impl Db {
    #[instrument]
    pub async fn stake_value(&self, stake_address_hex: String) -> Result<BigDecimal> {
        let query = query(stake_address_hex);

        let rows: Vec<BigDecimal> = self.exec(move |conn| query.load(conn)).await?;

        Ok(rows.iter().sum())
    }

    #[instrument]
    pub fn stake_values(
        &self,
        slot_no: Option<SlotNo>,
        stake_addrs: &[String],
    ) -> Result<Vec<(String, BigDecimal)>> {
        todo!()
        // stake_addrs
        //     .iter()
        //     .map(|s| -> Result<(String, BigDecimal)> { Ok((s.to_string(), self.stake_value(s)?)) })
        //     .collect()
    }
}

fn query(stake_addr: String) -> impl DbQuery<'static, BigDecimal> {
    use crate::db::schema::{stake_address, tx_in, tx_out};
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
        .filter(stake_address::hash_raw.eq(decode(stake_addr, "hex")))
}
