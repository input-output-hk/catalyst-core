use std::collections::HashMap;
use std::path::PathBuf;

use postgres::fallible_iterator::FallibleIterator;
use postgres::Client;

use crate::data::{NetworkId, RawRegistration, SignedRegistration, StakeKeyHex, TxId};
use crate::{InvalidRegistration, RegistrationError, SlotNo};
use cryptoxide::{blake2b::Blake2b, digest::Digest};

use nonempty::nonempty;

/// DB columns
const REG_TX_ID: usize = 2;
const REG_SLOT_NO: usize = 4;
const REG_JSON: usize = 6;
const REG_BIN: usize = 7;
const SIG_JSON: usize = 9;
const SIG_BIN: usize = 10;

/// Contains the most recent registration for each public stake address
pub type Valids = Vec<SignedRegistration>;

/// Registrations which failed cddl and or sig checks
pub type Invalids = Vec<InvalidRegistration>;

pub type StakeKeyHash = Vec<u8>;

///
/// Query gathers all possible registration transactions
/// Each registration is screened and marked: valid or invalid
///
/// # Errors
///
/// Any errors produced by the DB get returned.
///
pub fn filter_registrations(
    min_slot: SlotNo,
    max_slot: SlotNo,
    mut client: Client,
    network_id: NetworkId,
) -> Result<(Valids, Invalids), Box<dyn std::error::Error>> {
    let mut valids: Valids = vec![];
    let mut invalids: Invalids = vec![];

    let cddl = CddlConfig::new()?;

    let mut results = client.query_raw(
        "
        SELECT meta_table.id as reg_id,
        sig_table.id as sig_id,
        meta_table.tx_id as reg_tx_id,
        sig_table.tx_id as sig_tx_id,
        block.slot_no,
        meta_table.key as reg_key,
        meta_table.json as reg_json,
        meta_table.bytes as reg_bytes,
        sig_table.key as sig_key,
        sig_table.json as sig_json,
        sig_table.bytes as sig_bytes
     FROM (((tx_metadata AS meta_table INNER JOIN tx
        ON (tx.id = meta_table.tx_id)) INNER JOIN tx_metadata AS sig_table
       ON (sig_table.tx_id = meta_table.tx_id)) INNER JOIN block ON (block.id = tx.block_id))
     WHERE ((((meta_table.key = 61284) AND (sig_table.key = 61285))) AND
        ((block.slot_no >= $1) AND (block.slot_no <= $2))) ORDER BY meta_table.tx_id DESC;
    ",
        &[
            &i64::try_from(min_slot.0).unwrap(),
            &i64::try_from(max_slot.0).unwrap(),
        ],
    )?;

    while let Some(row) = results.next()? {
        // Here we can use a threadpool with a size == number of cores.
        // We can process each row in parallel using this pool.
        if valids.len() % 100 == 0 {
            info!(
                "registrations processed {:?}",
                valids.len() + invalids.len()
            );
        }

        // registration tx_id
        let tx_id: i64 = row.get(REG_TX_ID);
        let slot: i64 = row.get(REG_SLOT_NO);

        // The raw registration data from the database.
        let rawreg = RawRegistration {
            json_reg: row.get(REG_JSON),
            json_sig: row.get(SIG_JSON),
            bin_reg: row.get(REG_BIN),
            bin_sig: row.get(SIG_BIN),
            tx_id: TxId(tx_id as u64),
            slot: slot as u64,
        };

        // deserialize the raw Binary CBOR.
        let reg = match rawreg.to_signed(&cddl, network_id) {
            Err(err) => {
                invalids.push(InvalidRegistration {
                    registration: None,
                    errors: nonempty![RegistrationError::CborDeserializationFailed {
                        err: format!("Failed to deserialize Registration CBOR: {}", err),
                    }],
                });
                continue;
            }
            Ok(reg) => reg,
        };

        match reg.validate_signature_bin(rawreg.bin_reg.clone()) {
            Ok(_) => valids.push(reg),
            Err(err) => {
                invalids.push(InvalidRegistration {
                    registration: Some(reg),
                    errors: nonempty![RegistrationError::SignatureError {
                        err: format!("Signature validation failure: {}", err),
                    }],
                });
                continue;
            }
        }
    }

    Ok((latest_registrations(&valids, &mut invalids), invalids))
}

/// Each stake key can have multiple registrations, the latest must be identified and the rest partitioned
pub fn latest_registrations(valids: &Valids, invalids: &mut Invalids) -> Valids {
    let mut latest: HashMap<StakeKeyHex, SignedRegistration> = HashMap::new();

    for valid in valids {
        if let Some((_stake_key, current)) = latest.get_key_value(&valid.registration.stake_key) {
            if valid.registration.nonce > current.registration.nonce {
                invalids.push(InvalidRegistration {
                    registration: Some(current.clone()),
                    errors: nonempty![RegistrationError::ObsoleteRegistration {}],
                });
                latest.insert(valid.registration.stake_key.clone(), valid.clone());
            } else {
                invalids.push(InvalidRegistration {
                    registration: Some(valid.clone()),
                    errors: nonempty![RegistrationError::ObsoleteRegistration {}],
                });
            }
        } else {
            latest.insert(valid.registration.stake_key.clone(), valid.clone());
        }
    }

    latest.values().cloned().collect()
}

/// The registration has a 32 byte "Stake Public Key".  This is the raw ED25519 public key of the stake address.
/// To calculate the Voting power, you need the stake key hash. Encoded in Cardano format.
pub fn stake_key_hash(key: &StakeKeyHex, network: NetworkId) -> StakeKeyHash {
    let bytes = &key.0 .0;

    let mut digest = [0u8; 28];
    let mut context = Blake2b::new(28);
    context.input(&bytes);
    context.result(&mut digest);

    let e0 = hex::decode("e0").unwrap();
    let e1 = hex::decode("e1").unwrap();

    let ctx = match network {
        NetworkId::Testnet => [e0, digest.to_vec()].concat(),
        NetworkId::Mainnet => [e1, digest.to_vec()].concat(),
    };

    ctx
}

pub fn validate_reg_cddl(
    bin_reg: &Vec<u8>,
    cddl_config: &CddlConfig,
) -> Result<(), RegistrationError> {
    cddl::validate_cbor_from_slice(&cddl_config._61284, &bin_reg, None).map_err(|err| {
        RegistrationError::CddlParsingFailed {
            err: format!("reg bytes does not match 61284 spec: {}", err),
        }
    })?;

    Ok(())
}

pub fn validate_sig_cddl(
    bin_sig: &Vec<u8>,
    cddl_config: &CddlConfig,
) -> Result<(), RegistrationError> {
    cddl::validate_cbor_from_slice(&cddl_config._61285, &bin_sig, None).map_err(|err| {
        RegistrationError::CddlParsingFailed {
            err: format!("sig bytes does not match 61285 spec: {}", err),
        }
    })?;

    Ok(())
}

/// Cddl schema:
/// https://cips.cardano.org/cips/cip36/schema.cddl
pub struct CddlConfig {
    _61284: String,
    _61285: String,
}

impl CddlConfig {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let cddl_61284: String = String::from_utf8(cddl_file("61284.cddl".to_string())?)?;

        let cddl_61285: String = String::from_utf8(cddl_file("61285.cddl".to_string())?)?;

        Ok(CddlConfig {
            _61284: cddl_61284,
            _61285: cddl_61285,
        })
    }
}

fn cddl_file(file: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let dir = std::env::current_dir()?;
    let path = format!(
        "{}/src/voting-tools-rs/src/par/{}",
        dir.as_path().display().to_string(),
        file
    );

    let raw = std::fs::read(PathBuf::from(path))?;
    Ok(raw)
}
