use crate::data_provider::DataProvider;
use cardano_serialization_lib::address::{Address, NetworkInfo, RewardAddress};
use cardano_serialization_lib::chain_crypto::Blake2b256;
use cardano_serialization_lib::crypto::Ed25519Signature;
use cardano_serialization_lib::metadata::{
    GeneralTransactionMetadata, MetadataList, MetadataMap, TransactionMetadatum,
};
use cardano_serialization_lib::utils::{BigNum, Int};
use cardano_serialization_lib::{address::StakeCredential, crypto::PublicKey};
use color_eyre::eyre::{bail, eyre};
use color_eyre::eyre::{Context, Result};
use microtype::Microtype;

use crate::model::{network_info, Output, Reg, SlotNo, StakeKeyHex, StakePubKey, TestnetMagic};
use crate::VotingPowerSource;

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
    db: &dyn DataProvider,
    min_slot: Option<SlotNo>,
    max_slot: Option<SlotNo>,
    testnet_magic: Option<TestnetMagic>,
) -> Result<Vec<Output>> {
    let network_info = network_info(testnet_magic);
    let regs = db.vote_registrations(min_slot, max_slot)?;

    debug!("found {} possible registrations", regs.len());

    let stake_addrs = regs
        .iter()
        .filter_map(|reg| get_stake_address(&reg.metadata.stake_key, &network_info).ok())
        .collect::<Vec<_>>();

    let values = db.stake_values(&stake_addrs)?;

    let reg_voting_power = regs
        .into_iter()
        .filter_map(|reg| {
            if let Err(e) = reg.check_valid() {
                warn!("invalid reg on tx: '{}': {e}", reg.tx_id);
                return None;
            }

            debug!("registration on tx '{}' is valid", reg.tx_id);

            let stake_addr = get_stake_address(&reg.metadata.stake_key, &network_info).ok()?;
            let voting_power = values.get(&stake_addr as &str)?.clone();

            let output = Output {
                tx_id: reg.tx_id,
                voting_power_source: reg.metadata.voting_power_source.clone(),
                rewards_address: reg.metadata.rewards_addr.clone(),
                stake_public_key: reg.metadata.stake_key.convert(),
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
    stake_vkey_hex: &StakeKeyHex,
    network: &NetworkInfo,
) -> Result<String> {
    let stake_vkey_hex = stake_vkey_hex.trim_start_matches("0x");
    // TODO support stake extended keys
    if stake_vkey_hex.len() == 128 {
        // TODO: why is this bad? can we give a better error here?
        bail!("stake_vkey has length 128");
    } else {
        // Convert hex to public key
        let hex = hex::decode(stake_vkey_hex)?;
        let pub_key = PublicKey::from_bytes(&hex).map_err(|_| eyre!(""))?;
        let cred = StakeCredential::from_keyhash(&pub_key.hash());
        let stake_addr = RewardAddress::new(network.network_id(), &cred).to_address();
        let stake_addr_bytes = stake_addr.to_bytes();
        let stake_addr_bytes_hex = hex::encode(stake_addr_bytes);
        Ok(stake_addr_bytes_hex)
    }
}

impl Reg {
    /// Checks if this registration is valid
    ///
    /// Returns `Result` rather than `bool` to allow structured failures
    #[instrument]
    fn check_valid(&self) -> Result<()> {
        let stake_vkey = self.metadata.stake_key.trim_start_matches("0x");
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
        let delegations = match self.metadata.voting_power_source.clone() {
            VotingPowerSource::Delegated(delegations) => {
                let mut metadata_list = MetadataList::new();
                for (delegation, weight) in delegations {
                    let mut inner_metadata_list = MetadataList::new();
                    inner_metadata_list.add(
                        &TransactionMetadatum::new_bytes(hex::decode(
                            delegation.trim_start_matches("0x"),
                        )?)
                        .map_err(|e| eyre!(format!("cannot decode delegation key, due to: {e}")))?,
                    );
                    inner_metadata_list.add(&TransactionMetadatum::new_int(&Int::new(
                        &BigNum::from(weight),
                    )));
                    metadata_list.add(&TransactionMetadatum::new_list(&inner_metadata_list));
                }
                TransactionMetadatum::new_list(&metadata_list)
            }
            VotingPowerSource::Legacy(k) => {
                let bytes = hex::decode(k.trim_start_matches("0x"))?;
                TransactionMetadatum::new_bytes(bytes).map_err(|e| {
                    eyre!(format!(
                        "cannot decode legacy delegation key, due to: {}",
                        e
                    ))
                })?
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
        meta_map.insert(
            &TransactionMetadatum::new_int(&Int::new_i32(5)),
            &TransactionMetadatum::new_int(&Int::new(&BigNum::from(self.metadata.purpose.0))),
        );

        let mut meta = GeneralTransactionMetadata::new();
        meta.insert(
            &BigNum::from(61284u32),
            &TransactionMetadatum::new_map(&meta_map),
        );

        let meta_bytes = meta.to_bytes();
        let meta_bytes_hash = Blake2b256::new(&meta_bytes);

        // Get signature from rego
        let sig_str = self.signature.inner.0.clone().split_off(2);
        let sig =
            Ed25519Signature::from_hex(&sig_str).map_err(|e| eyre!("invalid ed25519 sig: {e}"))?;

        match pub_key.verify(meta_bytes_hash.as_hash_bytes(), &sig) {
            true => Ok(()),
            false => Err(eyre!("signature verification failed")),
        }
    }
}

#[cfg(test)]
mod tests {
    use cardano_serialization_lib::metadata::{GeneralTransactionMetadata, MetadataJsonSchema};

    use crate::test_api::MockDbProvider;
    use crate::DataProvider;
    use mainnet_lib::{
        BlockBuilder, CardanoWallet, GeneralTransactionMetadataInfo, InMemoryDbSync,
        TransactionBuilder,
    };

    #[test]
    pub fn nami_wallet_tx() {
        let cardano_wallet = CardanoWallet::new(1);

        let metadata_string = r#"{
            "61284":  {"1":[["0xf83870fde8c07e0552040cb4b005d63d58cd5f6450454f253844df7f76764a35",1]],"2":"0xebd231995f6bdbe6a1582625d1c291eed374fc7baab03feab6701ec21395180e","3":"0xe0b6d6e47f7d683a90bf4c638d337e95aaebcc9584188ca817a8691604","4":14150366,"5":0},
            "61285": {"1":"0x826c2edf51906865c27f0f8217faac6d2301636bad867d8182176025952ab76ebb8180f31a36370ab2229c55ad6f4137c9c8745e89bedc933727c56d351b5307"}
         }"#;

        let root_metadata = GeneralTransactionMetadata::from_json_string(
            metadata_string,
            MetadataJsonSchema::BasicConversions,
        )
        .unwrap();

        let transaction = TransactionBuilder::build_transaction_with_metadata(
            &cardano_wallet.address().to_address(),
            cardano_wallet.stake(),
            &root_metadata,
        );

        let mut db_sync = InMemoryDbSync::default();
        db_sync.on_block_propagation(&BlockBuilder::next_block(None, &vec![transaction]));

        let regs = MockDbProvider::from(db_sync)
            .vote_registrations(None, None)
            .unwrap();

        assert!(regs[0].check_valid().is_ok());
    }
}
