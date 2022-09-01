
const NO_SLOT_NO_STATEMENT: &str = r#"

    CREATE TEMPORARY TABLE IF NOT EXISTS utxo_snapshot AS (SELECT tx_out.*, stake_address.hash_raw AS stake_credential FROM tx_out LEFT OUTER JOIN tx_in ON tx_out.tx_id = tx_in.tx_out_id AND tx_out.index = tx_in.tx_out_index INNER JOIN stake_address ON stake_address.id = tx_out.stake_address_id WHERE tx_in.tx_in_id IS NULL);

    CREATE INDEX IF NOT EXISTS utxo_snapshot_stake_credential ON utxo_snapshot(stake_credential);

    ANALYZE utxo_snapshot;
"#;

// impl Db {
//     #[instrument]
//     pub async fn create_snapshot_table(&self, slot_no: Option<SlotNo>) -> Result<SnapshotTable> {
//         let mut conn = self.conn().await?;
//
//         if let Some(slot_no) = slot_no {
//             let slot_no = i64::try_from(slot_no.0)?;
//
//             let tx = conn.transaction().await?;
//
//             let tx_out_snapshot = tx
//                 .prepare(
//                     "CREATE TEMPORARY TABLE IF NOT EXISTS tx_out_snapshot AS (
//                 SELECT tx_out.*,
//                 stake_address.hash_raw AS stake_credential
//                   FROM tx_out
//                   INNER JOIN tx ON tx_out.tx_id = tx.id
//                   INNER JOIN block ON tx.block_id = block.id
//                   INNER JOIN stake_address ON stake_address.id = tx_out.stake_address_id
//                   WHERE block.slot_no <= $1);",
//                 )
//                 .await?;
//
//             let tx_in_snapshot = tx
//                 .prepare(
//                     "CREATE TEMPORARY TABLE IF NOT EXISTS tx_in_snapshot AS (
//             SELECT tx_in.* FROM tx_in
//               INNER JOIN tx ON tx_in.tx_in_id = tx.id
//               INNER JOIN block ON tx.block_id = block.id
//               WHERE block.slot_no <= $1);",
//                 )
//                 .await?;
//
//             let utxo_snapshot = "CREATE TEMPORARY TABLE IF NOT EXISTS utxo_snapshot AS (
//             SELECT tx_out_snapshot.* FROM tx_out_snapshot
//               LEFT OUTER JOIN tx_in_snapshot
//                 ON tx_out_snapshot.tx_id = tx_in_snapshot.tx_out_id
//                 AND tx_out_snapshot.index = tx_in_snapshot.tx_out_index
//               WHERE tx_in_snapshot.tx_in_id IS NULL);";
//
//             let stake_credential_index = "CREATE INDEX IF NOT EXISTS utxo_snapshot_stake_credential ON utxo_snapshot(stake_credential);";
//             let analyze_tx_out_snapshot = "ANALYZE tx_out_snapshot;";
//             let analyze_tx_in_snapshot = "ANALYZE tx_in_snapshot;";
//             let analyze_utxo_snapshot = "ANALYZE utxo_snapshot;";
//
//             tx.execute(&tx_out_snapshot, &[&slot_no]).await?;
//             tx.execute(&tx_in_snapshot, &[&slot_no]).await?;
//             tx.batch_execute(utxo_snapshot).await?;
//             tx.batch_execute(stake_credential_index).await?;
//             tx.batch_execute(analyze_tx_out_snapshot).await?;
//             tx.batch_execute(analyze_tx_in_snapshot).await?;
//             tx.batch_execute(analyze_utxo_snapshot).await?;
//         } else {
//             conn.batch_execute(NO_SLOT_NO_STATEMENT).await?;
//         }
//
//         Ok(SnapshotTable { _internal: () })
//     }
// }

/// Witness type to prove that the table was created
///
/// By requiring this type as an argument to a function, we guarantee that this function cannot run
/// without creating the table
#[derive(Clone, Copy)]
pub struct SnapshotTable {
    _internal: (), // private field prevent creation outside this module
}
