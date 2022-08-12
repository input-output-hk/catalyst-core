use postgres::{Client, Error, NoTls};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use cardano_serialization_lib::metadata::MetadataMap;
use cardano_serialization_lib::chain_crypto::Blake2b256;
use std::collections::HashMap;
use cardano_serialization_lib::address::NetworkInfo;
use cardano_serialization_lib::utils::BigNum;
use cardano_serialization_lib::address::Address;
use cardano_serialization_lib::utils::Int;
use cardano_serialization_lib::crypto::Ed25519Signature;
use cardano_serialization_lib::chain_crypto;
use cardano_serialization_lib::address::{RewardAddress, StakeCredential};
use cardano_serialization_lib::crypto::Ed25519KeyHash;
use cardano_serialization_lib::crypto::PublicKey;
use cardano_serialization_lib::utils::to_bytes;
use chain_core::property::FromStr;
use hex;
use rust_decimal::prelude::*;
use compare::{Compare, natural};
use std::cmp::Ordering::{Less, Equal, Greater};
use cardano_serialization_lib::metadata::TransactionMetadatum;
use cardano_serialization_lib::metadata::GeneralTransactionMetadata;


#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[derive(Clone)]
enum Delegations {
    Legacy(String),
    Delegated(Vec<(String, u32)>),
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
struct RegoMetadata {
    #[serde(rename = "1")]
    delegations: Delegations,
    #[serde(rename = "2")]
    stake_vkey: String,
    #[serde(rename = "3")]
    rewards_addr: String,
    #[serde(rename = "4")]
    slot: u64,
    #[serde(rename = "5")]
    #[serde(default = "catalystPurpose")]
    purpose: u64,
}

fn catalystPurpose() -> u64 {
    return 0;
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
struct RegoSignature {
    #[serde(rename = "1")]
    signature: String,
}

#[derive(Clone)]
struct Rego {
    tx_id: i64,
    metadata: RegoMetadata,
    signature: RegoSignature,
}

#[derive(Serialize, Deserialize)]
struct Output {
    delegations: Delegations,
    rewards_address: String,
    stake_public_key: String,
    voting_power: u64,
    voting_purpose: u64,
}

// fn main() -> Result<(), Error> {
//     // Connect to db-sync database
//     let mut client = Client::connect(
//         "postgresql://cexplorer@cexplorer?host=/var/run/postgresql",
//         NoTls,
//     )?;

//     // TODO parse slot number from command line
//     let regos: Vec<Rego> = query_vote_registrations(&mut client, None)?;

//     // let regos_latest: Vec<Rego> = filter_latest_registrations(regos);

//     let only_regos : Vec<Rego> = regos
//         .into_iter()
//         .filter(|r| r.metadata.stake_vkey == "0x000d28a9abc7b2f384bd39f6b7d49b631caec75df63f33066fe23f83824e85a7")
//         .collect();
//     let only_rego = &only_regos[2];
//     let only_rego_meta = serde_json::to_string_pretty(&only_rego.metadata).unwrap();
//     let only_rego_sig = serde_json::to_string_pretty(&only_rego.signature).unwrap();
//     // println!("{only_rego_meta}");
//     // println!("{only_rego_sig}");
//     mk_meta(only_rego);

//     Ok(())
// }

fn main() -> Result<(), Error> {
    // Connect to db-sync database
    let mut client = Client::connect(
        "postgresql://cexplorer@cexplorer?host=/var/run/postgresql",
        NoTls,
    )?;

    // TODO parse slot number from command line
    let regos: Vec<Rego> = query_vote_registrations(&mut client, None)?;

    // TODO get network id from cmd line
    let network_id = NetworkInfo::mainnet().network_id();

    let regos_valid: Vec<Rego> = filter_valid_registrations(regos, network_id);

    let regos_latest: Vec<Rego> = filter_latest_registrations(regos_valid);

    let mut rego_voting_power = Vec::new();
    mk_stake_snapshot_table(&mut client, None);
    for rego in regos_latest {
        let stake_address = get_stake_address(&rego.metadata.stake_vkey, network_id);
        match stake_address {
            None => {}
            Some(stk) => {
                let voting_power = query_stake_value(&mut client, &stk).unwrap();
                rego_voting_power.push((rego, voting_power));
            }
        }
    }

    let mut output: Vec<Output> = Vec::new();
    for (rego, voting_power) in rego_voting_power {
        let entry = Output {
            delegations: rego.metadata.delegations.clone(),
            rewards_address: rego.metadata.rewards_addr.clone(),
            stake_public_key: rego.metadata.stake_vkey.clone(),
            voting_power: voting_power.clone(),
            voting_purpose: rego.metadata.purpose,
        };
        output.push(entry);
    }

    let output_json = serde_json::to_string_pretty(&output).unwrap();
    println!("{output_json}");

    Ok(())
}

// 0b0000 for testnet OR NetworkInfo::testnet().network_id()
// 0b0001 for mainnet OR NetworkInfo::mainnet().network_id()
// Network magic doesn't matter
fn get_stake_address(stake_vkey_hex: &String, network_id: u8) -> Option<String> {
    // Remove initial '0x' from string
    let stake_vkey_hex_only = stake_vkey_hex.clone().split_off(2);
    // TODO support stake extended keys
    if stake_vkey_hex_only.len() == 128 {
        None
    } else {
        // Convert hex to public key
        let pub_key = PublicKey::from_bytes(&hex::decode(&stake_vkey_hex_only).unwrap()).unwrap();
        let cred = StakeCredential::from_keyhash(&pub_key.hash());
        let cred_bytes = to_bytes(&cred);
        let cred_bytes_hex = hex::encode(&cred_bytes);
        let stake_addr : Address = RewardAddress::new(network_id, &cred).to_address();
        let stake_addr_bytes = stake_addr.to_bytes();
        let stake_addr_bytes_hex = hex::encode(&stake_addr_bytes);
        Some(stake_addr_bytes_hex)
    }
}

fn query_vote_registrations(
    client: &mut Client,
    m_slot_no: Option<u64>,
) -> Result<Vec<Rego>, Error> {
    let sql_base = "WITH meta_table AS (select tx_id, json AS metadata from tx_metadata where key = '61284') , sig_table AS (select tx_id, json AS signature from tx_metadata where key = '61285') SELECT tx.hash,tx_id,metadata,signature FROM meta_table INNER JOIN tx ON tx.id = meta_table.tx_id INNER JOIN sig_table USING(tx_id)";

    let query = match m_slot_no {
        Some(slot_no) => format!("{sql_base} INNER JOIN block ON block.id = tx.block_id WHERE block.slot_no {slot_no} ORDER BY metadata -> '4' ASC;"),
        None => format!("{sql_base} ORDER BY metadata -> '4' ASC;"),
    };

    let mut regos = Vec::new();

    for row in (*client).query(&query, &[])? {
        // TODO: ignore bad JSON, although the fact that all data is valid
        // TxMetadata will protect us for now.
        let metadata_json = row.try_get::<usize, serde_json::Value>(2)?;
        let signature_json = row.try_get::<usize, serde_json::Value>(3)?;

        match (
            serde_json::from_value(metadata_json),
            serde_json::from_value(signature_json),
        ) {
            (Ok(metadata), Ok(signature)) => {
                let rego = Rego {
                    // txHash = row.get(0); <- We don't actually use this
                    tx_id: row.get(1),
                    metadata: metadata,
                    signature: signature,
                };

                regos.push(rego);
            }
            (_, _) => {}
        }
    }

    Ok(regos)
}



fn filter_valid_registrations(regos: Vec<Rego>, network_id: u8) -> Vec<Rego> {
    let mut regos_valid = Vec::new();

    for rego in regos {
        if mk_meta(&rego) { regos_valid.push(rego) } else { }
    }

    regos_valid
}

fn mk_meta(rego: &Rego) -> bool {
    // Remove initial '0x' from string
    let stake_vkey_hex_only = rego.metadata.stake_vkey.clone().split_off(2);
    if stake_vkey_hex_only.len() == 128 { false } else {
        let pub_key = PublicKey::from_bytes(&hex::decode(&stake_vkey_hex_only).unwrap()).unwrap();

        // Get rewards address
        let rewards_addr : Address = Address::from_bytes(hex::decode(&rego.metadata.rewards_addr.clone().split_off(2)).unwrap()).unwrap();
        let m_rewards_stake_addr : Option<RewardAddress> = RewardAddress::from_address(&rewards_addr);

        match m_rewards_stake_addr {
            None => { false }
            Some(rewards_stake_addr) => {
                let mut meta_whole : GeneralTransactionMetadata = GeneralTransactionMetadata::new();

                // Translate registration to Cardano metadata type so we can serialize it correctly
                let mut meta_map : MetadataMap = MetadataMap::new();
                let delegations = match rego.metadata.delegations.clone() {
                    Delegations::Delegated(ds) => TransactionMetadatum::new_text("foo".to_string()).unwrap(),
                    Delegations::Legacy(k) => {
                        let bytes = hex::decode(k.clone().split_off(2)).unwrap();
                        TransactionMetadatum::new_bytes(bytes).unwrap()
                    }
                };
                meta_map.insert(&TransactionMetadatum::new_int(&Int::new_i32(1)), &delegations);
                meta_map.insert(&TransactionMetadatum::new_int(&Int::new_i32(2)), &TransactionMetadatum::new_bytes(pub_key.as_bytes()).unwrap());
                meta_map.insert(&TransactionMetadatum::new_int(&Int::new_i32(3)), &TransactionMetadatum::new_bytes(rewards_addr.to_bytes()).unwrap());
                meta_map.insert(&TransactionMetadatum::new_int(&Int::new_i32(4)), &TransactionMetadatum::new_int(&Int::new(&BigNum::from(rego.metadata.slot))));

                let mut meta = GeneralTransactionMetadata::new();
                meta.insert(&BigNum::from(61284 as u32), &TransactionMetadatum::new_map(&meta_map));

                let meta_bytes = meta.to_bytes();
                let meta_bytes_hex = hex::encode(&meta_bytes);
                let meta_bytes_hash = Blake2b256::new(&meta_bytes);

                // Get signature from rego
                let sig_str = rego.signature.signature.clone().split_off(2);
                match Ed25519Signature::from_hex(&sig_str) {
                    Err(e) => { false },
                    Ok(sig) => {
                        if pub_key.verify(meta_bytes_hash.as_hash_bytes(), &sig) {
                            true
                        } else {
                            false
                        }
                    }
                }
            }
        }
    }
}

fn filter_latest_registrations(regos: Vec<Rego>) -> Vec<Rego> {
    // Group the registrations by stake key (each stake key may have one valid registration)
    let mut m = HashMap::new();
    for rego in regos {
        let stake_key = rego.metadata.stake_vkey.clone();
        m.entry(stake_key).or_insert_with(Vec::new).push(rego)
    }
    // Find the regos with the highest slot number, and of those, choose the
    // lowest txid.
    let mut latest_regos = Vec::new();
    for (_, stake_regos) in m {
        let latest = stake_regos.iter().fold(stake_regos[0].clone(), |acc, rego| {
            let cmp = natural();
            match cmp.compare(&rego.metadata.slot, &acc.metadata.slot) {
                // If the slot number is less, it's not a newer registration.
                Less => acc,
                // If the slot number is greater, it's a newer registration.
                Greater => rego.clone(),
                // If the slot number is equal, choose the one with the lower tx id.
                Equal => if rego.tx_id < acc.tx_id { rego.clone() } else { acc },
            }
        });
        latest_regos.push(latest.clone())
    }
    latest_regos
}

fn mk_stake_snapshot_table(client: &mut Client, m_slot_no: Option<u64>) -> Result<(), Error> {
    match m_slot_no {
        None => {
            let stake_credential_index = "CREATE INDEX IF NOT EXISTS utxo_snapshot_stake_credential ON utxo_snapshot(stake_credential);";
            let analyze_table = "ANALYZE utxo_snapshot;";
            let utxo_snapshot = "CREATE TEMPORARY TABLE IF NOT EXISTS utxo_snapshot AS (SELECT tx_out.*, stake_address.hash_raw AS stake_credential FROM tx_out LEFT OUTER JOIN tx_in ON tx_out.tx_id = tx_in.tx_out_id AND tx_out.index = tx_in.tx_out_index INNER JOIN stake_address ON stake_address.id = tx_out.stake_address_id WHERE tx_in.tx_in_id IS NULL);";
            (*client)
                .batch_execute(&format!(
                    "{utxo_snapshot} {stake_credential_index} {analyze_table}"
                ))
                .map(|_x| ())
        }
        Some(slot_no) => {
            let tx_out_snapshot = format!(
                "CREATE TEMPORARY TABLE IF NOT EXISTS tx_out_snapshot AS (
                SELECT tx_out.*,
                stake_address.hash_raw AS stake_credential
                  FROM tx_out
                  INNER JOIN tx ON tx_out.tx_id = tx.id
                  INNER JOIN block ON tx.block_id = block.id
                  INNER JOIN stake_address ON stake_address.id = tx_out.stake_address_id
                  WHERE block.slot_no <= {slot_no});"
            );
            let tx_in_snapshot = format!(
                "CREATE TEMPORARY TABLE IF NOT EXISTS tx_in_snapshot AS (
            SELECT tx_in.* FROM tx_in
              INNER JOIN tx ON tx_in.tx_in_id = tx.id
              INNER JOIN block ON tx.block_id = block.id
              WHERE block.slot_no <= {slot_no});"
            );
            let utxo_snapshot = "CREATE TEMPORARY TABLE IF NOT EXISTS utxo_snapshot AS (
            SELECT tx_out_snapshot.* FROM tx_out_snapshot
              LEFT OUTER JOIN tx_in_snapshot
                ON tx_out_snapshot.tx_id = tx_in_snapshot.tx_out_id
                AND tx_out_snapshot.index = tx_in_snapshot.tx_out_index
              WHERE tx_in_snapshot.tx_in_id IS NULL);";
            let stake_credential_index = "CREATE INDEX IF NOT EXISTS utxo_snapshot_stake_credential ON utxo_snapshot(stake_credential);";
            let analyze_tx_out_snapshot = "ANALYZE tx_out_snapshot;";
            let analyze_tx_in_snapshot = "ANALYZE tx_in_snapshot;";
            let analyze_utxo_snapshot = "ANALYZE utxo_snapshot;";
            (*client).batch_execute(&format!("{tx_out_snapshot} {analyze_tx_out_snapshot} {tx_in_snapshot} {analyze_tx_in_snapshot} {utxo_snapshot} {stake_credential_index} {analyze_utxo_snapshot}")).map(|_x| ())
        }
    }
}

// Precondition: mk_stake_snapshot_table has been run:
fn query_stake_value(client: &mut Client, stake_address_hex: &String) -> Result<u64, Error> {
    let stake_query_sql = format!("SELECT utxo_snapshot.value FROM utxo_snapshot WHERE stake_credential = decode('{stake_address_hex}', 'hex');");
    // Don't do SUM in the query, lovelace is a bounded integer type defined by
    // cardano-db-sync, unless you perform a conversion to an unbounded type,
    // it will overflow if the SUM exceeds the max value of a lovelace db
    // type.
    let mut values = Vec::new();
    for row in (*client).query(&stake_query_sql, &[])? {
        let db_val: Decimal = row.get(0);
        let val = match u64::try_from(db_val) {
            Err(_e) => 0,
            Ok(unsigned) => unsigned,
        };
        values.push(val);
    }
    Ok(values.iter().sum())
}

fn query_stake_values(
    client: &mut Client,
    m_slot_no: Option<u64>,
    stake_addresses_hex: Vec<String>,
) -> Result<Vec<(String, u64)>, Error> {
    mk_stake_snapshot_table(client, m_slot_no)?;

    let mut stake_values = Vec::new();
    for stake_address_hex in stake_addresses_hex {
        let value = query_stake_value(client, &stake_address_hex)?;
        stake_values.push((stake_address_hex, value));
    }
    Ok(stake_values)
}
