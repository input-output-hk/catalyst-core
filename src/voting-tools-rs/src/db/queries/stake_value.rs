use color_eyre::Result;
use futures::future::join_all;
use rust_decimal::Decimal;

use crate::{db::Db, model::SlotNo};

use super::snapshot_table::SnapshotTable;

impl Db {
    #[instrument(skip(_table))]
    pub async fn stake_value(&self, _table: SnapshotTable, stake_address_hex: &str) -> Result<u64> {
        let conn = self.conn().await?;
        let rows = conn.query("SELECT utxo_snapshot.value FROM utxo_snapshot WHERE stake_credential = decode('$1', 'hex');", &[&stake_address_hex]).await?;

        let sum = rows
            .into_iter()
            .map(|row| {
                let value: Decimal = row.get(0);
                u64::try_from(value).unwrap_or(0)
            })
            .sum();

        Ok(sum)
    }

    #[instrument]
    pub async fn stake_values(
        &self,
        slot_no: Option<SlotNo>,
        stake_addrs: &[&str],
    ) -> Result<Vec<(String, u64)>> {
        let table = self.create_snapshot_table(slot_no).await?;

        let futures = stake_addrs.iter().map(|s| self.stake_value(table, s));

        // join_all allows us to poll all the futures in parallel
        let results = join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        let result = results
            .into_iter()
            .enumerate()
            .map(|(index, sum)| (stake_addrs[index].to_string(), sum))
            .collect();

        Ok(result)
    }
}
