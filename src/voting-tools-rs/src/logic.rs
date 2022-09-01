use std::collections::HashMap;

use cardano_serialization_lib::address::{Address, NetworkInfo, RewardAddress};
use cardano_serialization_lib::chain_crypto::Blake2b256;
use cardano_serialization_lib::crypto::Ed25519Signature;
use cardano_serialization_lib::metadata::{
    GeneralTransactionMetadata, MetadataMap, TransactionMetadatum,
};
use cardano_serialization_lib::utils::{BigNum, Int};
use cardano_serialization_lib::{address::StakeCredential, crypto::PublicKey};
use color_eyre::eyre::Result;
use color_eyre::eyre::{bail, eyre};

use crate::config::DbConfig;
use crate::model::{Delegations, Output, Rego, SlotNo, StakeVKey, TestnetMagic};

/// Calculate voter registration info by connecting to a db-sync instance
#[instrument]
pub fn run(
    db: DbConfig,
    slot_no: Option<SlotNo>,
    testnet_magic: Option<TestnetMagic>,
) -> Result<Vec<Output>> {
    todo!()
    // let network_info = network_info(testnet_magic);
    // let db = Db::connect(db).await?;
    //
    // let regos = db.vote_registrations(slot_no)?;
    //
    // let regos = regos.into_iter().filter(|r| r.is_valid().is_ok());
    // let regos = filter_latest_registrations(regos);
    //
    // let mut rego_voting_power = Vec::with_capacity(regos.len());
    // let table = db.create_snapshot_table(slot_no).await?;
    //
    // for rego in regos {
    //     let stake_address = get_stake_address(&rego.metadata.stake_vkey, &network_info);
    //     match stake_address {
    //         Err(_) => {}
    //         Ok(stk) => {
    //             let voting_power = db.stake_value(table, &stk).await?;
    //             rego_voting_power.push((rego, voting_power));
    //         }
    //     }
    // }
    //
    // let mut output = Vec::with_capacity(rego_voting_power.len());
    //
    // for (rego, voting_power) in rego_voting_power {
    //     let entry = Output {
    //         delegations: rego.metadata.delegations.clone(),
    //         rewards_address: rego.metadata.rewards_addr.clone(),
    //         stake_public_key: rego.metadata.stake_vkey.clone().convert(),
    //         voting_power: voting_power.into(),
    //         voting_purpose: rego.metadata.purpose,
    //     };
    //     output.push(entry);
    // }
    //
    // Ok(output)
}

fn filter_latest_registrations(regos: impl IntoIterator<Item = Rego>) -> Vec<Rego> {
    // Group the registrations by stake key (each stake key may have one valid registration)
    let mut m = HashMap::new();
    for rego in regos {
        let stake_key = rego.metadata.stake_vkey.clone();
        m.entry(stake_key).or_insert_with(Vec::new).push(rego);
    }
    // Find the regos with the highest slot number, and of those, choose the
    // lowest txid.
    let mut latest_regos = Vec::new();
    for (_, stake_regos) in m {
        let latest = stake_regos
            .iter()
            .fold(stake_regos[0].clone(), |acc, rego| {
                use std::cmp::Ordering::*;
                match &rego.metadata.slot.cmp(&acc.metadata.slot) {
                    // If the slot number is less, it's not a newer registration.
                    Less => acc,
                    // If the slot number is greater, it's a newer registration.
                    Greater => rego.clone(),
                    // If the slot number is equal, choose the one with the lower tx id.
                    Equal => {
                        if rego.tx_id < acc.tx_id {
                            rego.clone()
                        } else {
                            acc
                        }
                    }
                }
            });
        latest_regos.push(latest.clone())
    }
    latest_regos
}

#[instrument]
pub(crate) fn get_stake_address(
    stake_vkey_hex: &StakeVKey,
    network_info: &NetworkInfo,
) -> Result<String> {
    // Remove initial '0x' from string
    if !stake_vkey_hex.starts_with("0x") {
        warn!("stake_vkey_hex doesn't start with `0x`");
    }

    let stake_vkey_hex_only = stake_vkey_hex.trim_start_matches("0x");
    // TODO support stake extended keys
    if stake_vkey_hex_only.len() == 128 {
        // TODO: why is this bad? can we give a better error here?
        bail!("stake_vkey has length 128");
    } else {
        // Convert hex to public key
        let hex = hex::decode(&stake_vkey_hex_only)?;
        let pub_key = PublicKey::from_bytes(&hex).map_err(|_| eyre!(""))?;
        let cred = StakeCredential::from_keyhash(&pub_key.hash());
        let stake_addr: Address = RewardAddress::new(network_info.network_id(), &cred).to_address();
        let stake_addr_bytes = stake_addr.to_bytes();
        let stake_addr_bytes_hex = hex::encode(&stake_addr_bytes);
        Ok(stake_addr_bytes_hex)
    }
}

impl Rego {
    /// Checks if this registration is valid
    ///
    /// Returns `Result` rather than `bool` to allow structured failures
    #[instrument]
    fn is_valid(&self) -> Result<()> {
        let stake_vkey = self.metadata.stake_vkey.trim_start_matches("0x");
        if stake_vkey.len() == 128 {
            // TODO: why is this bad? can we give a better error here?
            bail!("stake_vkey has length 128");
        }

        let hex = hex::decode(stake_vkey)?;
        let pub_key =
            // this error doesn't impl `std::err::Error`
            PublicKey::from_bytes(&hex).map_err(|e| eyre!("error decoding public key: {e}"))?;

        // Get rewards address
        let rewards_addr: Address =
            Address::from_bytes(hex::decode(&*self.metadata.rewards_addr)?).unwrap();

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
