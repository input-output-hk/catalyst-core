use std::collections::HashMap;
use std::path::PathBuf;

use postgres::Client;
use std::cmp::Reverse;

use crate::data::{NetworkId, Registration, Signature, SignedRegistration, StakeKeyHex, TxId};
use crate::validation::hash;
use crate::{InvalidRegistration, RegistrationError, VotingPowerArgs};
use cardano_serialization_lib::chain_crypto::Ed25519;
use cardano_serialization_lib::chain_crypto::{
    AsymmetricPublicKey, Verification, VerificationAlgorithm,
};
use cryptoxide::{blake2b::Blake2b, digest::Digest};

use nonempty::nonempty;

use serde_json::{self, json};

/// DB columns
const REG_TX_ID: usize = 2;
const REG_JSON: usize = 6;
const REG_BIN: usize = 7;
const SIG_JSON: usize = 9;
const SIG_BIN: usize = 10;

/// All registrations found by the query where the Signature check passes.
/// Signature check done against the raw binary data of the registration
pub type ValidSigs = HashMap<StakeKeyHex, Vec<SignedRegistration>>;

/// Contains the most recent registration for each public stake address
/// Obsolete registrations extracted to invalid TX list
pub type Valids = HashMap<StakeKeyHex, SignedRegistration>;

/// Registrations which failed cddl and or sig checks
pub type Invalids = Vec<InvalidRegistration>;

/// Obsolete registrations extracted to invalid TX list
pub type InvalidTxs = Vec<SignedRegistration>;

///
/// Query gathers all possible registration transactions
/// Each registration is screened and marked: valid or invalid
pub fn filter_registrations(
    args: VotingPowerArgs,
    mut client: Client,
) -> Result<(Valids, Invalids, InvalidTxs), Box<dyn std::error::Error>> {
    let mut valids: ValidSigs = HashMap::new();
    let mut invalids: Invalids = vec![];

    let cddl = CddlConfig::new()?;

    for row in client.query(
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
            &i64::try_from(args.min_slot.unwrap().0).unwrap(),
            &i64::try_from(args.max_slot.unwrap().0).unwrap(),
        ],
    )? {
        // cip 36: 61284 json
        let json_reg: serde_json::Value = row.get(REG_JSON);

        // cip 36: 61285 json
        let json_sig: serde_json::Value = row.get(SIG_JSON);

        // cip 36: 61284 raw binary
        let bin_reg: Vec<u8> = row.get(REG_BIN);

        // cip 36: 61285 raw binary
        let bin_sig: Vec<u8> = row.get(SIG_BIN);

        // registration tx_id
        let tx_id: i64 = row.get(REG_TX_ID);
        let tx_id: TxId = TxId(tx_id as u64);

        validate_registration(
            &mut invalids,
            &mut valids,
            bin_reg,
            bin_sig,
            tx_id,
            json_reg,
            json_sig,
            &cddl,
        );
    }

    let (valids, invalid_txs) = latest_registrations(valids);
    Ok((valids, invalids, invalid_txs))
}

pub fn validate_registration(
    invalids: &mut Invalids,
    valid_sigs: &mut ValidSigs,
    bin_reg: Vec<u8>,
    bin_sig: Vec<u8>,
    tx_id: TxId,
    json_reg: serde_json::Value,
    json_sig: serde_json::Value,
    cddl_config: &CddlConfig,
) {
    // validate cddl: 61824
    if let Err(err) = validate_reg_cddl(bin_reg.clone(), &cddl_config) {
        invalids.push(InvalidRegistration {
            registration: None,
            errors: nonempty![err],
        });
        return;
    }

    // validate cddl: 61825
    if let Err(err) = validate_sig_cddl(bin_sig.clone(), &cddl_config) {
        invalids.push(InvalidRegistration {
            registration: None,
            errors: nonempty![err],
        });
        return;
    }

    let r: Registration = match serde_json::from_value(json_reg) {
        Ok(r) => r,
        Err(err) => {
            invalids.push(InvalidRegistration {
                registration: None,
                errors: nonempty![RegistrationError::RegistrationFormat(
                    json!(err.to_string())
                )],
            });
            return;
        }
    };

    let s: Signature = match serde_json::from_value(json_sig) {
        Ok(s) => s,
        Err(err) => {
            invalids.push(InvalidRegistration {
                registration: None,
                errors: nonempty![RegistrationError::SignatureFormat(json!(err.to_string()))],
            });
            return;
        }
    };

    let sr = SignedRegistration {
        registration: r,
        signature: s,
        tx_id: tx_id,
    };

    match validate_signature_bin(&sr.registration.clone(), &sr.signature.clone(), bin_reg) {
        Ok(_) => valid_sigs
            .entry(sr.clone().registration.stake_key)
            .or_insert(Vec::new())
            .push(sr),
        Err(err) => {
            invalids.push(InvalidRegistration {
                registration: Some(sr),
                errors: nonempty![err],
            });
            return;
        }
    }
}

/// Each stake key can have multiple registrations, the latest must be identified and the rest partitioned
pub fn latest_registrations(
    valid_sigs: ValidSigs,
) -> (HashMap<StakeKeyHex, SignedRegistration>, InvalidTxs) {
    let mut valids: HashMap<StakeKeyHex, SignedRegistration> = HashMap::new();
    let mut invalids: InvalidTxs = vec![];

    // Find the latest Registration Record for each stake address key
    for (key, mut value) in valid_sigs {
        value.sort_by_key(|r| Reverse(r.registration.nonce));

        // The latest transaction is defined as the transaction with the largest nonce field
        let latest_tx = value[0].clone();

        // The "obsolete" transactions should be added to the invalid transactions list.
        for r in value.drain(1..) {
            invalids.push(r);
        }

        valids.insert(key, latest_tx);
    }

    (valids, invalids)
}

/// The registration has a 32 byte "Stake Public Key".  This is the raw ED25519 public key of the stake address.
/// To calculate the Voting power, you need the stake key hash. Encoded in Cardano format.
pub fn stake_key_hash(key: &StakeKeyHex, network: NetworkId) -> String {
    let bytes = &key.0 .0;

    let mut digest = [0u8; 28];
    let mut context = Blake2b::new(28);
    context.input(&bytes);
    context.result(&mut digest);

    let digest_hex = hex::encode(digest);

    let ctx = match network {
        NetworkId::Mainnet => format!("E1{}", digest_hex),
        NetworkId::Testnet => format!("EO{}", digest_hex),
    };
    ctx
}

/// The signature is generated by:
///  - CBOR encoding the registration
///  - blake2b-256 hashing those bytes
///  - signing the hash with the private key used to generate the stake key
fn validate_signature_bin(
    registration: &Registration,
    Signature { inner: sig }: &Signature,
    bin_reg: Vec<u8>,
) -> Result<(), RegistrationError> {
    let bytes = bin_reg;
    let hash_bytes = hash::hash(&bytes);

    let pub_key = Ed25519::public_from_binary(registration.stake_key.as_ref())
        .map_err(|e| RegistrationError::StakePublicKeyError { err: e.to_string() })?;
    let sig = Ed25519::signature_from_bytes(sig.as_ref())
        .map_err(|e| RegistrationError::SignatureError { err: e.to_string() })?;

    match Ed25519::verify_bytes(&pub_key, &sig, &hash_bytes) {
        Verification::Success => Ok(()),
        Verification::Failed => Err(RegistrationError::MismatchedSignature { hash_bytes }),
    }
}

pub fn validate_reg_cddl(
    bin_reg: Vec<u8>,
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
    bin_sig: Vec<u8>,
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
