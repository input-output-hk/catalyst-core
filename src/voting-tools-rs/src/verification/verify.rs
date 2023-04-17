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
                        err: format!("Failed to deserialize Registration CBOR: {}", err),
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
            Ok(_) => valids.push(reg),
            Err(err) => {
                invalids.push(InvalidRegistration {
                    spec_61284: Some(prefix_hex(&rawreg.bin_reg)),
                    spec_61285: Some(prefix_hex(&rawreg.bin_sig)),
                    registration: Some(reg),
                    errors: nonempty![RegistrationError::SignatureError {
                        err: format!("Signature validation failure: {}", err),
                    }],
                    registration_bad_bin: None,
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
    let prefix_hex = format!("{:x}", rewards_address_prefix);

    // 0 or 1 are valid addrs in the following cases:
    // type = 0x0 -  network = 0
    // type = 0x0 -  network = 1
    match network {
        NetworkId::Mainnet => (),
        NetworkId::Testnet => {
            if prefix_hex == *"0" || prefix_hex == *"1" {
                return true;
            }
        }
    }

    // All other cases should have type and network id present
    if prefix_hex.len() != 2 {
        return false;
    }

    // First nibble identifies type
    let address_type = prefix_hex.chars().nth(0).unwrap();
    // second nibble identifies network id
    let network_id = prefix_hex.chars().nth(1).unwrap();

    // Valid addrs: 0x0?, 0x1?, 0x2?, 0x3?, 0x4?, 0x5?, 0x6?, 0x7?, 0xE?, 0xF?.
    let valid_addrs = 0..8;
    let addr = address_type.to_digit(16).unwrap();
    if !valid_addrs.contains(&addr) && addr != 14 && addr != 15 {
        error!("invalid rewards addr prefix {:?} {:?}", prefix_hex, addr);
        return false;
    }

    let net_id = network_id.to_digit(16).unwrap();

    match network {
        NetworkId::Mainnet => {
            if net_id != 1 {
                return false;
            }
        }
        NetworkId::Testnet => {
            if net_id != 0 {
                return false;
            }
        }
    }
    true
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
            err: format!("reg bytes does not match 61284 spec: {}", err),
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
            err: format!("sig bytes does not match 61285 spec: {}", err),
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

/// encoding of hex strings with a 0x prefix.
#[must_use]
pub fn prefix_hex(b: &[u8]) -> String {
    format!("0x{}", hex::encode(b))
}

#[cfg(test)]
mod tests {
    use crate::{data::NetworkId, verify::is_valid_rewards_address};
    use bitvec::prelude::*;

    #[test]
    pub fn test_rewards_addr_permuations() {
        // Valid addrs: 0x0?, 0x1?, 0x2?, 0x3?, 0x4?, 0x5?, 0x6?, 0x7?, 0xE?, 0xF?.

        // preprod - network id: 0
        let network_0 = NetworkId::Testnet;

        // prod - network id: 1
        let network_1 = NetworkId::Mainnet;

        // 0xE - 0
        let mut e0 = bitvec![u8, Msb0;];
        e0.push(true);
        e0.push(true);
        e0.push(true);
        e0.push(false);
        e0.push(false);
        e0.push(false);
        e0.push(false);
        e0.push(false);
        assert!(is_valid_rewards_address(&e0.as_raw_slice()[0], network_0));

        // 0xE - 1
        let mut e1 = bitvec![u8, Msb0;];
        e1.push(true);
        e1.push(true);
        e1.push(true);
        e1.push(false);
        e1.push(false);
        e1.push(false);
        e1.push(false);
        e1.push(true);
        assert!(is_valid_rewards_address(&e1.as_raw_slice()[0], network_1));

        // 0xF - 0
        let mut f0 = bitvec![u8, Msb0;];
        f0.push(true);
        f0.push(true);
        f0.push(true);
        f0.push(true);
        f0.push(false);
        f0.push(false);
        f0.push(false);
        f0.push(false);
        assert!(is_valid_rewards_address(&f0.as_raw_slice()[0], network_0));

        // 0xF - 1
        let mut f1 = bitvec![u8, Msb0;];
        f1.push(true);
        f1.push(true);
        f1.push(true);
        f1.push(false);
        f1.push(false);
        f1.push(false);
        f1.push(false);
        f1.push(true);
        assert!(is_valid_rewards_address(&f1.as_raw_slice()[0], network_1));

        // 0x0 - 0
        let mut o00 = bitvec![u8, Msb0;];
        o00.push(false);
        o00.push(false);
        o00.push(false);
        o00.push(false);
        o00.push(false);
        o00.push(false);
        o00.push(false);
        o00.push(false);
        assert!(is_valid_rewards_address(&o00.as_raw_slice()[0], network_0));

        // 0x0 - 1
        let mut o01 = bitvec![u8, Msb0;];
        o01.push(false);
        o01.push(false);
        o01.push(false);
        o01.push(false);
        o01.push(false);
        o01.push(false);
        o01.push(false);
        o01.push(true);

        // 0x1 - 0
        let mut o10 = bitvec![u8, Msb0;];
        o10.push(false);
        o10.push(false);
        o10.push(false);
        o10.push(true);
        o10.push(false);
        o10.push(false);
        o10.push(false);
        o10.push(false);
        assert!(is_valid_rewards_address(&o10.as_raw_slice()[0], network_0));

        // 0x1 - 1
        let mut o11 = bitvec![u8, Msb0;];
        o11.push(false);
        o11.push(false);
        o11.push(false);
        o11.push(true);
        o11.push(false);
        o11.push(false);
        o11.push(false);
        o11.push(true);
        assert!(is_valid_rewards_address(&o11.as_raw_slice()[0], network_1));

        // 0x2 - 0
        let mut o20 = bitvec![u8, Msb0;];
        o20.push(false);
        o20.push(false);
        o20.push(true);
        o20.push(false);
        o20.push(false);
        o20.push(false);
        o20.push(false);
        o20.push(false);
        assert!(is_valid_rewards_address(&o20.as_raw_slice()[0], network_0));

        // 0x2 - 1
        let mut o21 = bitvec![u8, Msb0;];
        o21.push(false);
        o21.push(false);
        o21.push(true);
        o21.push(false);
        o21.push(false);
        o21.push(false);
        o21.push(false);
        o21.push(true);
        assert!(is_valid_rewards_address(&o21.as_raw_slice()[0], network_1));

        // 0x3 - 0
        let mut o30 = bitvec![u8, Msb0;];
        o30.push(false);
        o30.push(false);
        o30.push(true);
        o30.push(true);
        o30.push(false);
        o30.push(false);
        o30.push(false);
        o30.push(false);
        assert!(is_valid_rewards_address(&o30.as_raw_slice()[0], network_0));

        // 0x3 - 1
        let mut o31 = bitvec![u8, Msb0;];
        o31.push(false);
        o31.push(false);
        o31.push(true);
        o31.push(true);
        o31.push(false);
        o31.push(false);
        o31.push(false);
        o31.push(true);
        assert!(is_valid_rewards_address(&o21.as_raw_slice()[0], network_1));

        // 0x4 - 0
        let mut o40 = bitvec![u8, Msb0;];
        o40.push(false);
        o40.push(true);
        o40.push(false);
        o40.push(false);
        o40.push(false);
        o40.push(false);
        o40.push(false);
        o40.push(false);
        assert!(is_valid_rewards_address(&o40.as_raw_slice()[0], network_0));

        // 0x4 - 1
        let mut o41 = bitvec![u8, Msb0;];
        o41.push(false);
        o41.push(true);
        o41.push(false);
        o41.push(false);
        o41.push(false);
        o41.push(false);
        o41.push(false);
        o41.push(true);
        assert!(is_valid_rewards_address(&o41.as_raw_slice()[0], network_1));

        // 0x5 - 0
        let mut o50 = bitvec![u8, Msb0;];
        o50.push(false);
        o50.push(true);
        o50.push(false);
        o50.push(true);
        o50.push(false);
        o50.push(false);
        o50.push(false);
        o50.push(false);
        assert!(is_valid_rewards_address(&o50.as_raw_slice()[0], network_0));

        // 0x5 - 1
        let mut o51 = bitvec![u8, Msb0;];
        o51.push(false);
        o51.push(true);
        o51.push(false);
        o51.push(true);
        o51.push(false);
        o51.push(false);
        o51.push(false);
        o51.push(true);
        assert!(is_valid_rewards_address(&o51.as_raw_slice()[0], network_1));

        // 0x6 - 0
        let mut o60 = bitvec![u8, Msb0;];
        o60.push(false);
        o60.push(true);
        o60.push(true);
        o60.push(false);
        o60.push(false);
        o60.push(false);
        o60.push(false);
        o60.push(false);
        assert!(is_valid_rewards_address(&o60.as_raw_slice()[0], network_0));

        // 0x6 - 1
        let mut o61 = bitvec![u8, Msb0;];
        o61.push(false);
        o61.push(true);
        o61.push(true);
        o61.push(false);
        o61.push(false);
        o61.push(false);
        o61.push(false);
        o61.push(true);
        assert!(is_valid_rewards_address(&o61.as_raw_slice()[0], network_1));

        // 0x7 - 0
        let mut o70 = bitvec![u8, Msb0;];
        o70.push(false);
        o70.push(true);
        o70.push(true);
        o70.push(true);
        o70.push(false);
        o70.push(false);
        o70.push(false);
        o70.push(false);
        assert!(is_valid_rewards_address(&o70.as_raw_slice()[0], network_0));

        // 0x7 - 1
        let mut o71 = bitvec![u8, Msb0;];
        o71.push(false);
        o71.push(true);
        o71.push(true);
        o71.push(true);
        o71.push(false);
        o71.push(false);
        o71.push(false);
        o71.push(true);

        assert!(is_valid_rewards_address(&o71.as_raw_slice()[0], network_1));

        // should fail

        // 0x9 - 0
        let mut o90 = bitvec![u8, Msb0;];
        o90.push(true);
        o90.push(false);
        o90.push(false);
        o90.push(true);
        o90.push(false);
        o90.push(false);
        o90.push(false);
        o90.push(false);
        assert_eq!(
            is_valid_rewards_address(&o90.as_raw_slice()[0], network_0),
            false
        );

        // 0x9 - 1
        let mut o91 = bitvec![u8, Msb0;];
        o91.push(true);
        o91.push(false);
        o91.push(false);
        o91.push(true);
        o91.push(false);
        o91.push(false);
        o91.push(false);
        o91.push(true);
        assert_eq!(
            is_valid_rewards_address(&o91.as_raw_slice()[0], network_1),
            false
        );
    }
}
