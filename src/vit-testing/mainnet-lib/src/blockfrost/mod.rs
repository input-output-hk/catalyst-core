use crate::wallet::GeneralTransactionMetadataInfo;
use crate::{REGISTRATION_METADATA_IDX, REGISTRATION_SIGNATURE_METADATA_IDX};
use blockfrost::{load, AddressTransaction, BlockFrostApi, BlockFrostSettings};
use cardano_serialization_lib::address::Address;
use cardano_serialization_lib::error::JsError;
use cardano_serialization_lib::metadata::{GeneralTransactionMetadata, MetadataJsonSchema};

/// Wrapper on Blockfrost api tailored for catalyst needs
pub struct CatalystBlockFrostApi {
    api: BlockFrostApi,
}

impl CatalystBlockFrostApi {
    /// build blockfrost sdk
    ///
    /// # Errors
    ///
    /// On blockfrost related error
    ///
    /// # Panics
    ///
    /// On internal blockfrost configuration error
    pub fn new() -> Result<CatalystBlockFrostApi, Error> {
        let configurations = load::configurations_from_env()?;
        let project_id = configurations["project_id"].as_str().unwrap();
        let settings = BlockFrostSettings::new().use_testnet();
        Ok(Self {
            api: BlockFrostApi::new(project_id, settings),
        })
    }

    /// Gets funds for given address
    ///
    /// # Errors
    ///
    /// On internal blockfrost error
    ///
    /// # Panics
    ///
    /// On blockfrost wrong internal ada representation
    pub async fn get_stake(&self, address: &Address) -> Result<u64, Error> {
        let address = self.api.accounts(&address.to_hex()).await?;
        //we trust blockfrost API
        Ok(address.controlled_amount.parse().unwrap())
    }

    async fn get_registration_tx(
        &self,
        tx: &AddressTransaction,
    ) -> Result<Option<GeneralTransactionMetadata>, Error> {
        let metadata = self.api.transactions_metadata(&tx.tx_hash).await?;

        let registration_part = metadata
            .iter()
            .find(|x| x.label == REGISTRATION_METADATA_IDX.to_string());
        let signature_part = metadata
            .iter()
            .find(|x| x.label == REGISTRATION_SIGNATURE_METADATA_IDX.to_string());

        Ok(
            if let (Some(registration), Some(signature)) = (registration_part, signature_part) {
                Some(GeneralTransactionMetadata::from_jsons(
                    registration.clone().json_metadata,
                    signature.clone().json_metadata,
                    MetadataJsonSchema::BasicConversions,
                )?)
            } else {
                None
            },
        )
    }

    /// Retrieve all registration metadata for address
    ///
    /// # Errors
    ///
    /// On internal blockfrost api error
    pub async fn list_registrations_for_address(
        &self,
        address: impl Into<String>,
    ) -> Result<Vec<GeneralTransactionMetadata>, Error> {
        let txs = self.api.addresses_transactions(&address.into()).await?;
        let mut output = vec![];
        for tx in txs {
            if let Some(metadata) = self.get_registration_tx(&tx).await? {
                output.push(metadata);
            }
        }
        Ok(output)
    }
}

/// Blockfrost api related Error
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// General blockfrost api error
    #[error("blockfrost")]
    Blockfrost(#[from] blockfrost::Error),
    /// Json conversion error
    #[error("blockfrost")]
    JsonConversion(#[from] crate::JsonConversionError),
    /// Cardano serialization lib error
    #[error("cardano serialization lib")]
    CardanoSerialization(#[from] JsError),
}
