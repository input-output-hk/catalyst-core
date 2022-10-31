use crate::data_provider::DataProvider;
use crate::model::{
    Delegations, Reg, RegoMetadata, RegoSignature, RewardsAddr, Signature, SlotNo, StakeVKey, TxId,
    VotingPurpose,
};
use bigdecimal::BigDecimal;
use cardano_serialization_lib::address::Address;
use cardano_serialization_lib::crypto::{Ed25519Signature, PublicKey};
use cardano_serialization_lib::metadata::{TransactionMetadatum, TransactionMetadatumLabel};
use cardano_serialization_lib::utils::Int;
use mainnet_lib::DbSyncInstance;
use std::collections::HashMap;
use std::str::FromStr;

/// Mock db provider based on [`DbSyncInstance`] struct from [`mainnet_lib`] project.
/// In essence struct keep data in memory and provides query for voting tools logic
#[derive(Debug)]
pub struct MockDbProvider {
    db_sync_instance: DbSyncInstance,
}

impl DataProvider for MockDbProvider {
    fn vote_registrations(
        &self,
        lower: Option<SlotNo>,
        upper: Option<SlotNo>,
    ) -> color_eyre::Result<Vec<Reg>> {
        let lower = if let Some(lower) = lower {
            #[allow(clippy::cast_sign_loss)]
            Some(lower.into_i64()? as u64)
        } else {
            None
        };

        let upper = if let Some(upper) = upper {
            #[allow(clippy::cast_sign_loss)]
            Some(upper.into_i64()? as u64)
        } else {
            None
        };

        let filtered_regs = self
            .db_sync_instance
            .query_voting_transactions_with_bounds(lower, upper);

        let mut tx_id = 0;

        Ok(filtered_regs
            .iter()
            .map(|(_block0, registrations)| {
                registrations
                    .iter()
                    .map(|r| {
                        let metadata = r.get(&TransactionMetadatumLabel::from(61284u32)).unwrap();
                        let metadata_map = metadata.as_map().unwrap();

                        let signature_metadata =
                            r.get(&TransactionMetadatumLabel::from(61285u32)).unwrap();
                        let signature_metadata_map = signature_metadata.as_map().unwrap();

                        let delegations = {
                            let metadata = metadata_map
                                .get(&TransactionMetadatum::new_int(&Int::new_i32(1)))
                                .unwrap();

                            if let Ok(data) = metadata.as_bytes() {
                                Delegations::Legacy(hex::encode(data))
                            } else {
                                Delegations::Delegated(vec![])
                            }
                        };

                        tx_id += 1;

                        let pub_key = PublicKey::from_bytes(
                            &metadata_map
                                .get(&TransactionMetadatum::new_int(&Int::new_i32(2)))
                                .unwrap()
                                .as_bytes()
                                .unwrap(),
                        )
                        .unwrap();
                        let rewards_address = Address::from_bytes(
                            metadata_map
                                .get(&TransactionMetadatum::new_int(&Int::new_i32(3)))
                                .unwrap()
                                .as_bytes()
                                .unwrap(),
                        )
                        .unwrap();
                        let sig = Ed25519Signature::from_bytes(
                            signature_metadata_map
                                .get(&TransactionMetadatum::new_int(&Int::new_i32(1)))
                                .unwrap()
                                .as_bytes()
                                .unwrap(),
                        )
                        .unwrap();

                        Reg {
                            tx_id: TxId::from(tx_id),
                            metadata: RegoMetadata {
                                delegations,
                                stake_vkey: StakeVKey(pub_key.to_hex()),
                                rewards_addr: RewardsAddr(rewards_address.to_hex()),
                                slot: SlotNo(
                                    u64::from_str(
                                        &metadata_map
                                            .get(&TransactionMetadatum::new_int(&Int::new_i32(4)))
                                            .unwrap()
                                            .as_int()
                                            .unwrap()
                                            .to_str(),
                                    )
                                    .unwrap(),
                                ),
                                purpose: VotingPurpose(0),
                            },
                            signature: RegoSignature {
                                signature: Signature(format!("0x{}", sig.to_hex())),
                            },
                        }
                    })
                    .collect()
            })
            .fold(vec![], |mut acc, sub_vec: Vec<Reg>| {
                acc.extend(sub_vec);
                acc
            }))
    }

    fn stake_values<'a>(
        &self,
        stake_addrs: &'a [String],
    ) -> color_eyre::Result<HashMap<&'a str, BigDecimal>> {
        Ok(stake_addrs
            .iter()
            .map(|addr| {
                (
                    addr.as_str(),
                    BigDecimal::from(*self.db_sync_instance.stakes().get(addr).unwrap_or(&0u64)),
                )
            })
            .collect())
    }
}

impl From<DbSyncInstance> for MockDbProvider {
    fn from(db_sync: DbSyncInstance) -> Self {
        Self {
            db_sync_instance: db_sync,
        }
    }
}
