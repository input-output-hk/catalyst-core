use rust_decimal::Decimal;

use dashmap::DashMap;
use postgres::{fallible_iterator::FallibleIterator, Client};

/// DB columns
const STAKE_CREDENTIAL: usize = 0; // BYTEA
const STAKED_ADA: usize = 1; // NUMERIC

///
/// Get ALL UTXO for all possible Stake Addresses.
/// Given a maximum slot number
///
/// # Errors
///
/// Any errors produced by the DB get returned.
///
pub fn staked_utxo_ada(
    max_slot: i64,
    client: &mut Client,
) -> Result<DashMap<Vec<u8>, u128>, Box<dyn std::error::Error>> {
    info!("executing tx out statement");

    let tx_out = format!(
        "CREATE OR REPLACE TEMPORARY VIEW tx_out_snapshot AS (
        SELECT tx_out.*, stake_address.hash_raw AS stake_credential FROM tx_out
        INNER JOIN tx ON tx_out.tx_id = tx.id
        INNER JOIN block ON tx.block_id = block.id
        INNER JOIN stake_address ON stake_address.id = tx_out.stake_address_id
    WHERE block.slot_no <= {max_slot} );"
    );

    client.execute(&tx_out, &[])?;

    info!("executing tx in statement");

    let tx_in = format!(
        "CREATE OR REPLACE TEMPORARY VIEW tx_in_snapshot AS (
            SELECT tx_in.* FROM tx_in
                  INNER JOIN tx ON tx_in.tx_in_id = tx.id
                  INNER JOIN block ON tx.block_id = block.id
            WHERE block.slot_no <= {max_slot});"
    );

    client.execute(&tx_in, &[])?;

    info!("executing utxo snap statement");

    let utxo_snap = "CREATE OR REPLACE TEMPORARY VIEW utxo_snapshot AS (SELECT tx_out_snapshot.* FROM tx_out_snapshot
        LEFT OUTER JOIN tx_in_snapshot ON
        tx_out_snapshot.tx_id = tx_in_snapshot.tx_out_id AND
        tx_out_snapshot.index = tx_in_snapshot.tx_out_index
        WHERE tx_in_snapshot.tx_in_id IS NULL);".to_string();

    client.execute(&utxo_snap, &[])?;

    info!("executing stake credential statement");

    let params: [String; 0] = [];
    let mut results =
        client.query_raw("SELECT stake_credential, value from utxo_snapshot;", params)?;

    let result = DashMap::new();

    let mut processing_record = 0;
    while let Some(row) = results.next()? {
        let stake_hash: Vec<u8> = row.get(STAKE_CREDENTIAL);

        // https://github.com/sfackler/rust-postgres/issues/119
        let staked_ada: Decimal = row.get(STAKED_ADA);

        let staked_ada = rust_decimal::prelude::ToPrimitive::to_u128(&staked_ada).unwrap();

        *result.entry(stake_hash.clone()).or_insert_with(|| 0) += staked_ada;

        if processing_record % 1000 == 0 {
            info!("{:?} records processed", processing_record);
        }
        processing_record += 1;
    }

    Ok(result)
}
