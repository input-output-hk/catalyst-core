use std::collections::HashMap;

use dashmap::DashMap;
use postgres::fallible_iterator::FallibleIterator;
use postgres::Client;

use crate::data::{NetworkId, RawRegistration, SignedRegistration, StakeKeyHex, TxId};
use crate::{InvalidRegistration, RegistrationCorruptedBin, RegistrationError, SlotNo};
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

/// `Network_id` + Blake2b-224( Stake Public Key )
pub type StakeKeyHash = Vec<u8>;

/// Unregistered voters stake
pub type Unregistered = DashMap<Vec<u8>, u128>;

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
    cip_36_multidelegations: bool,
) -> Result<(Valids, Invalids), Box<dyn std::error::Error>> {
    let mut valids: Valids = vec![];
    let mut invalids: Invalids = vec![];

    let cddl = CddlConfig::new();

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
        if valids.len() % 1000 == 0 {
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
                    spec_61284: Some(prefix_hex(&rawreg.bin_reg)),
                    spec_61285: Some(prefix_hex(&rawreg.bin_sig)),
                    registration: None,
                    errors: nonempty![RegistrationError::CborDeserializationFailed {
                        err: format!("Failed to deserialize Registration CBOR: {err}"),
                    }],
                    registration_bad_bin: Some(RegistrationCorruptedBin {
                        tx_id: TxId(tx_id as u64),
                        slot: slot as u64,
                    }),
                });
                continue;
            }
            Ok(reg) => reg,
        };

        match reg.validate_signature_bin(rawreg.bin_reg.clone()) {
            Ok(_) => (),
            Err(err) => {
                invalids.push(InvalidRegistration {
                    spec_61284: Some(prefix_hex(&rawreg.bin_reg)),
                    spec_61285: Some(prefix_hex(&rawreg.bin_sig)),
                    registration: Some(reg),
                    errors: nonempty![RegistrationError::SignatureError {
                        err: format!("Signature validation failure: {err}"),
                    }],
                    registration_bad_bin: None,
                });
                continue;
            }
        }

        match reg.validate_multi_delegation(cip_36_multidelegations) {
            Ok(_) => (),
            Err(err) => {
                invalids.push(InvalidRegistration {
                    spec_61284: Some(prefix_hex(&rawreg.bin_reg)),
                    spec_61285: Some(prefix_hex(&rawreg.bin_sig)),
                    registration: Some(reg),
                    errors: nonempty![err],
                    registration_bad_bin: None,
                });
                continue;
            }
        }

        valids.push(reg);
    }

    Ok((latest_registrations(&valids, &mut invalids), invalids))
}

/// Each stake key can have multiple registrations, the latest must be identified and the rest partitioned
pub fn latest_registrations(valids: &Valids, invalids: &mut Invalids) -> Valids {
    let mut latest: HashMap<StakeKeyHex, SignedRegistration> = HashMap::new();

    for valid in valids {
        if let Some((_stake_key, current)) = latest.get_key_value(&valid.registration.stake_key) {
            if valid.registration.nonce > current.registration.nonce
                || ((valid.registration.nonce == current.registration.nonce)
                    && (valid.tx_id > current.tx_id))
            {
                invalids.push(InvalidRegistration {
                    spec_61284: None,
                    spec_61285: None,
                    registration: Some(current.clone()),
                    errors: nonempty![RegistrationError::ObsoleteRegistration {}],
                    registration_bad_bin: None,
                });
                latest.insert(valid.registration.stake_key.clone(), valid.clone());
            } else {
                invalids.push(InvalidRegistration {
                    spec_61284: None,
                    spec_61285: None,
                    registration: Some(valid.clone()),
                    errors: nonempty![RegistrationError::ObsoleteRegistration {}],
                    registration_bad_bin: None,
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
/// `Network_id` + Blake2b-224( Stake Public Key )
#[must_use]
pub fn stake_key_hash(key: &StakeKeyHex, network: NetworkId) -> StakeKeyHash {
    let bytes = &key.0 .0;

    let mut digest = [0u8; 28];
    let mut context = Blake2b::new(28);
    context.input(bytes);
    context.result(&mut digest);

    let e0 = hex::decode("e0").unwrap();
    let e1 = hex::decode("e1").unwrap();

    match network {
        NetworkId::Testnet => [e0, digest.to_vec()].concat(),
        NetworkId::Mainnet => [e1, digest.to_vec()].concat(),
    }
}

/// Reward addresses start with a single header byte identifying their type and the network,
/// followed by 28 bytes of payload identifying either a stake key hash or a script hash.
/// Function accepts this first header prefix byte.
/// Validates first nibble is within the address range: 0x0? - 0x7? + 0xE? , 0xF?
/// Validates second nibble matches network id: 0/1
#[must_use]
pub fn is_valid_rewards_address(rewards_address_prefix: &u8, network: NetworkId) -> bool {
    let addr_type = rewards_address_prefix >> 4 & 0xf;
    let addr_net = rewards_address_prefix & 0xf;

    // 0 or 1 are valid addrs in the following cases:
    // type = 0x0 -  Testnet network
    // type = 0x1 -  Mainnet network
    match network {
        NetworkId::Mainnet => {
            if addr_net != 1 {
                return false;
            }
        }
        NetworkId::Testnet => {
            if addr_net != 0 {
                return false;
            }
        }
    }

    // Valid addrs: 0x0?, 0x1?, 0x2?, 0x3?, 0x4?, 0x5?, 0x6?, 0x7?, 0xE?, 0xF?.
    let valid_addrs = [0, 1, 2, 3, 4, 5, 6, 7, 14, 15];
    valid_addrs.contains(&addr_type)
}

/// Validate raw registration binary against 61284 CDDL spec
///
/// # Errors
///
/// Failure will occur if parsed keys do not match CDDL spec
pub fn validate_reg_cddl(
    bin_reg: &[u8],
    cddl_config: &CddlConfig,
) -> Result<(), RegistrationError> {
    cddl::validate_cbor_from_slice(&cddl_config.spec_61284, bin_reg, None).map_err(|err| {
        RegistrationError::CddlParsingFailed {
            err: format!("reg bytes does not match 61284 spec: {err}"),
        }
    })?;

    Ok(())
}

/// Validate raw signature binary against 61285 CDDL spec
///
/// # Errors
///
/// Failure will occur if parsed keys do not match CDDL spec
pub fn validate_sig_cddl(
    bin_sig: &[u8],
    cddl_config: &CddlConfig,
) -> Result<(), RegistrationError> {
    cddl::validate_cbor_from_slice(&cddl_config.spec_61285, bin_sig, None).map_err(|err| {
        RegistrationError::CddlParsingFailed {
            err: format!("sig bytes does not match 61285 spec: {err}"),
        }
    })?;

    Ok(())
}

/// Cddl schema:
/// <https://cips.cardano.org/cips/cip36/schema.cddl>
pub struct CddlConfig {
    spec_61284: String,
    spec_61285: String,
}

impl CddlConfig {
    #[must_use]
    pub fn new() -> Self {
        let cddl_61284: String = include_str!("61284.cddl").to_string();
        let cddl_61285: String = include_str!("61285.cddl").to_string();

        CddlConfig {
            spec_61284: cddl_61284,
            spec_61285: cddl_61285,
        }
    }
}

impl Default for CddlConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// encoding of hex strings with a 0x prefix
#[must_use]
pub fn prefix_hex(b: &[u8]) -> String {
    format!("0x{}", hex::encode(b))
}

#[cfg(test)]
mod tests {
    use crate::{data::NetworkId, verify::is_valid_rewards_address};

    #[test]
    pub fn test_rewards_addr_permuations() {
        // Valid addrs: 0x0?, 0x1?, 0x2?, 0x3?, 0x4?, 0x5?, 0x6?, 0x7?, 0xE?, 0xF?.

        let valid_addr_types = vec![0, 1, 2, 3, 4, 5, 6, 7, 14, 15];

        for addr_type in valid_addr_types {
            let test_addr = addr_type << 4;
            assert!(is_valid_rewards_address(&test_addr, NetworkId::Testnet));
            assert!(!is_valid_rewards_address(&test_addr, NetworkId::Mainnet));

            let test_addr = addr_type << 4 | 1;
            assert!(!is_valid_rewards_address(&test_addr, NetworkId::Testnet));
            assert!(is_valid_rewards_address(&test_addr, NetworkId::Mainnet));
        }

        let invalid_addr_types = vec![8, 9, 10, 11, 12, 13];

        for addr_type in invalid_addr_types {
            let test_addr = addr_type << 4;
            assert!(!is_valid_rewards_address(&test_addr, NetworkId::Testnet));
            assert!(!is_valid_rewards_address(&test_addr, NetworkId::Mainnet));

            let test_addr = addr_type << 4 | 1;
            assert!(!is_valid_rewards_address(&test_addr, NetworkId::Testnet));
            assert!(!is_valid_rewards_address(&test_addr, NetworkId::Mainnet));
        }
    }
}
