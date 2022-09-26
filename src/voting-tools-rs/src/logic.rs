use cardano_serialization_lib::address::{Address, NetworkInfo, RewardAddress};
use cardano_serialization_lib::chain_crypto::Blake2b256;
use cardano_serialization_lib::crypto::Ed25519Signature;
use cardano_serialization_lib::metadata::{
    GeneralTransactionMetadata, MetadataMap, TransactionMetadatum,
};
use cardano_serialization_lib::utils::{BigNum, Int};
use cardano_serialization_lib::{address::StakeCredential, crypto::PublicKey};
use color_eyre::eyre::{bail, eyre};
use color_eyre::eyre::{Context, Result};
use microtype::Microtype;

use crate::model::{network_info, Delegations, Output, Reg, SlotNo, StakeVKey, TestnetMagic};
use crate::Db;

/// Calculate voting power info by querying a db-sync instance
///
/// Invalid registrations are silently ignored (e.g. if they contain bad/null JSON metadata, if
/// they have invalid signatures, etc).
///
/// If provided, `min_slot` and `max_slot` can  be used to constrain the time period to query. If
/// `None` they default to:
///  - `min_slot`: `0`
///  - `max_slot`: `i64::MAX`
///
/// Together they form an inclusive range (i.e. blocks with values equal to `min_slot` or `max_slot` are included)
///
/// # Errors
///
/// Returns an error if either of `lower` or `upper` doesn't fit in an `i64`
#[instrument]
pub fn voting_power(
    db: &Db,
    min_slot: Option<SlotNo>,
    max_slot: Option<SlotNo>,
    testnet_magic: Option<TestnetMagic>,
) -> Result<Vec<Output>> {
    let network_info = network_info(testnet_magic);
    let regs = db.vote_registrations(min_slot, max_slot)?;
    let stake_addrs = regs
        .iter()
        .filter_map(|reg| get_stake_address(&reg.metadata.stake_vkey, &network_info).ok())
        .collect::<Vec<_>>();

    let values = db.stake_values(&stake_addrs)?;

    let reg_voting_power = regs
        .into_iter()
        .filter_map(|reg| {
            if let Err(e) = reg.check_valid() {
                warn!("invalid reg: {e}");
                return None;
            }

            let stake_addr = get_stake_address(&reg.metadata.stake_vkey, &network_info).ok()?;
            let voting_power = values.get(&stake_addr as &str)?.clone();

            let output = Output {
                tx_id: reg.tx_id,
                delegations: reg.metadata.delegations.clone(),
                rewards_address: reg.metadata.rewards_addr.clone(),
                stake_public_key: reg.metadata.stake_vkey.convert(),
                voting_purpose: reg.metadata.purpose,
                voting_power,
            };

            Some(output)
        })
        .collect();

    Ok(reg_voting_power)
}

#[instrument]
pub(crate) fn get_stake_address(
    stake_vkey_hex: &StakeVKey,
    network: &NetworkInfo,
) -> Result<String> {
    let stake_vkey_hex = stake_vkey_hex.trim_start_matches("0x");
    // TODO support stake extended keys
    if stake_vkey_hex.len() == 128 {
        // TODO: why is this bad? can we give a better error here?
        bail!("stake_vkey has length 128");
    } else {
        // Convert hex to public key
        let hex = hex::decode(&stake_vkey_hex)?;
        let pub_key = PublicKey::from_bytes(&hex).map_err(|_| eyre!(""))?;
        let cred = StakeCredential::from_keyhash(&pub_key.hash());
        let stake_addr = RewardAddress::new(network.network_id(), &cred).to_address();
        let stake_addr_bytes = stake_addr.to_bytes();
        let stake_addr_bytes_hex = hex::encode(&stake_addr_bytes);
        Ok(stake_addr_bytes_hex)
    }
}

impl Reg {
    /// Checks if this registration is valid
    ///
    /// Returns `Result` rather than `bool` to allow structured failures
    #[instrument]
    fn check_valid(&self) -> Result<()> {
        let stake_vkey = self.metadata.stake_vkey.trim_start_matches("0x");
        if stake_vkey.len() == 128 {
            // TODO: why is this bad? can we give a better error here?
            bail!("stake_vkey has length 128");
        }

        let hex = hex::decode(stake_vkey).context("failed to decode hex")?;
        let pub_key =
            // this error doesn't impl `std::err::Error`
            PublicKey::from_bytes(&hex).map_err(|e| eyre!("error decoding public key: {e}"))?;

        // Get rewards address
        let rewards_addr = self.metadata.rewards_addr.trim_start_matches("0x");
        let rewards_addr: Address = Address::from_bytes(hex::decode(rewards_addr)?)
            .map_err(|_| eyre!("invalid address"))?;

        if RewardAddress::from_address(&rewards_addr).is_none() {
            bail!("invalid reward address");
        }

        // Translate registration to Cardano metadata type so we can serialize it correctly
        let mut meta_map: MetadataMap = MetadataMap::new();
        let delegations = match self.metadata.delegations.clone() {
            Delegations::Delegated(_) => {
                TransactionMetadatum::new_text("foo".to_string()).map_err(|_| eyre!("uh oh"))?
                // TODO: this seems weird
            }
            Delegations::Legacy(k) => {
                let bytes = hex::decode(k.trim_start_matches("0x"))?;
                TransactionMetadatum::new_bytes(bytes).unwrap()
            }
        };
        meta_map.insert(
            &TransactionMetadatum::new_int(&Int::new_i32(1)),
            &delegations,
        );
        meta_map.insert(
            &TransactionMetadatum::new_int(&Int::new_i32(2)),
            &TransactionMetadatum::new_bytes(pub_key.as_bytes()).unwrap(),
        );
        meta_map.insert(
            &TransactionMetadatum::new_int(&Int::new_i32(3)),
            &TransactionMetadatum::new_bytes(rewards_addr.to_bytes()).unwrap(),
        );
        meta_map.insert(
            &TransactionMetadatum::new_int(&Int::new_i32(4)),
            &TransactionMetadatum::new_int(&Int::new(&BigNum::from(self.metadata.slot.0))),
        );

        let mut meta = GeneralTransactionMetadata::new();
        meta.insert(
            &BigNum::from(61284u32),
            &TransactionMetadatum::new_map(&meta_map),
        );

        let meta_bytes = meta.to_bytes();
        let meta_bytes_hash = Blake2b256::new(&meta_bytes);

        // Get signature from rego
        let sig_str = self.signature.signature.0.clone().split_off(2);
        let sig =
            Ed25519Signature::from_hex(&sig_str).map_err(|e| eyre!("invalid ed25519 sig: {e}"))?;
        match pub_key.verify(meta_bytes_hash.as_hash_bytes(), &sig) {
            true => Ok(()),
            false => Err(eyre!("signature verification failed")),
        }
    }
}

