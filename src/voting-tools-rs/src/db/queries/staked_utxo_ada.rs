use postgres::fallible_iterator::FallibleIterator;
use postgres::Client;

///
/// Query gathers all possible registration transactions
/// Each registration is screened and marked: valid or invalid
///
/// # Errors
///
/// Any errors produced by the DB get returned.
///
pub fn staked_utxo_ada(
    max_slot: i64,
    client: &mut Client
) -> Result<(), Box<dyn std::error::Error>> {

    client.execute("
        CREATE OR REPLACE TEMPORARY VIEW tx_out_snapshot AS (
        SELECT tx_out.*, stake_address.hash_raw AS stake_credential FROM tx_out
            INNER JOIN tx ON tx_out.tx_id = tx.id
            INNER JOIN block ON tx.block_id = block.id
            INNER JOIN stake_address ON stake_address.id = tx_out.stake_address_id
        WHERE block.slot_no <= $1);
    ", &[&max_slot])?;

    client.execute("
        ANALYZE tx_out_snapshot;
    ",&[])?;

    client.execute("
        CREATE OR REPLACE TEMPORARY VIEW tx_in_snapshot AS (
        SELECT tx_in.* FROM tx_in
              INNER JOIN tx ON tx_in.tx_in_id = tx.id
              INNER JOIN block ON tx.block_id = block.id
        WHERE block.slot_no <= $1);
    ", &[&max_slot])?;

    client.execute("
        ANALYZE tx_in_snapshot;
    ",&[])?;

    client.execute("
        CREATE OR REPLACE TEMPORARY VIEW utxo_snapshot AS (
        SELECT tx_out_snapshot.* FROM tx_out_snapshot
            LEFT OUTER JOIN tx_in_snapshot ON
                tx_out_snapshot.tx_id = tx_in_snapshot.tx_out_id AND
                tx_out_snapshot.index = tx_in_snapshot.tx_out_index
            WHERE tx_in_snapshot.tx_in_id IS NULL);
    ", &[])?;

    client.execute("
        ANALYZE utxo_snapshot;
    ",&[])?;

    let mut results = client.query_raw("
            SELECT stake_credential, value from utxo_snapshot;
        ",
        &[
            max_slot,
        ],
    )?;

    while let Some(row) = results.next()? {
        info!("{:?}",row);
    }

    panic!("Stop");
    Ok(())
}
