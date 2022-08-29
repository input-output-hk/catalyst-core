use color_eyre::Result;
use serde_json::from_value;

use crate::{db::Db, model::Rego, model::SlotNo};

const SQL_BASE: &str = "
WITH meta_table AS (select tx_id, json AS metadata from tx_metadata where key = '61284') , sig_table AS (select tx_id, json AS signature from tx_metadata where key = '61285') SELECT tx.hash,tx_id,metadata,signature FROM meta_table INNER JOIN tx ON tx.id = meta_table.tx_id INNER JOIN sig_table USING(tx_id)
";

const SOME_SLOT_NO: &str = "
 INNER JOIN block ON block.id = tx.block_id WHERE block.slot_no $1 ORDER BY metadata -> '4' ASC;
";

const NO_SLOT_NO: &str = "
 ORDER BY metadata -> '4' ASC;
";

impl Db {
    pub async fn vote_registrations(&self, slot_no: Option<SlotNo>) -> Result<Vec<Rego>> {
        let conn = self.conn().await?;
        let rows = match slot_no {
            None => {
                let s = format!("{SQL_BASE} {NO_SLOT_NO}");
                conn.query(&s, &[]).await?
            }
            Some(slot_no) => {
                let slot_no = i64::try_from(slot_no.0).unwrap_or(0);
                let s = format!("{SQL_BASE} {SOME_SLOT_NO}");
                conn.query(&s, &[&slot_no]).await?
            }
        };

        let result = rows.into_iter().filter_map(|row| {
            let tx_id: i64 = row.get(1);
            let tx_id = u64::try_from(tx_id).ok()?;
            let metadata_json = row.try_get(2).ok()?;
            let signature_json = row.try_get(3).ok()?;

            let metadata = from_value(metadata_json);
            let signature = from_value(signature_json);

            match (metadata, signature) {
                (Ok(metadata), Ok(signature)) => Some(Rego {
                    tx_id: tx_id.into(),
                    metadata,
                    signature,
                }),
                _ => None,
            }
        });

        Ok(result.collect())
    }
}
