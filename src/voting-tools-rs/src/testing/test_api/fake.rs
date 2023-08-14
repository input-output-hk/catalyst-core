use crate::data::PubKey;
use crate::data::{
    Nonce, Registration, RewardsAddress, Signature, SignedRegistration, SlotNo, StakeKeyHex, TxId,
    VotingKey, VotingKeyHex, VotingPurpose,
};
use crate::data_provider::DataProvider;
use crate::verify::StakeKeyHash;
use crate::Sig;
use bigdecimal::{BigDecimal, FromPrimitive};
use cardano_serialization_lib::address::Address;
use cardano_serialization_lib::crypto::{Ed25519Signature, PublicKey};
use cardano_serialization_lib::utils::BigNum;
use dashmap::DashMap;
// This library is garbage and pulls in almost everything from cat-core.  We should NOT be using it for tests like this.
/*
use mainnet_lib::{
    InMemoryDbSync, METADATUM_1, METADATUM_2, METADATUM_3, METADATUM_4,
    REGISTRATION_METADATA_LABEL, REGISTRATION_METADATA_SIGNATURE_LABEL,
};
use std::str::FromStr;

/// Mock db provider based on [`InMemoryDbSync`] struct from [`mainnet_lib`] project.
///
/// This struct simulates a real db-sync instance, but keeps data in memory in a struct
#[derive(Debug)]
pub struct MockDbProvider {
    db_sync_instance: InMemoryDbSync,
}

impl DataProvider for MockDbProvider {
    fn vote_registrations(
        &self,
        lower: SlotNo,
        upper: SlotNo,
    ) -> color_eyre::Result<Vec<SignedRegistration>> {
        let filtered_regs = self
            .db_sync_instance
            .query_voting_transactions_with_bounds(*lower, *upper);

        let mut tx_id: u64 = 0;

        Ok(filtered_regs
            .values()
            .map(|registrations| {
                registrations
                    .iter()
                    .map(|r| {
                        let metadata = r.get(&REGISTRATION_METADATA_LABEL).unwrap();
                        let metadata_map = metadata.as_map().unwrap();

                        let signature_metadata =
                            r.get(&REGISTRATION_METADATA_SIGNATURE_LABEL).unwrap();
                        let signature_metadata_map = signature_metadata.as_map().unwrap();

                        let voting_key = {
                            let metadata = metadata_map.get(&METADATUM_1).unwrap();

                            if let Ok(data) = metadata.as_bytes() {
                                VotingKey::Direct(PubKey(data).into())
                            } else {
                                let mut delegations = Vec::new();
                                let delgation_list = metadata.as_list().unwrap();
                                for i in 0..delgation_list.len() {
                                    let inner_list = delgation_list.get(i).as_list().unwrap();

                                    let delegation = inner_list.get(0).as_bytes().unwrap();
                                    let delegation = PubKey(delegation);
                                    let weight = inner_list.get(1).as_int().unwrap();
                                    let weight =
                                        u32::from_i32(weight.as_i32_or_fail().unwrap()).unwrap();
                                    delegations.push((VotingKeyHex(delegation), weight.into()));
                                }

                                VotingKey::Delegated(delegations)
                            }
                        };

                        tx_id = tx_id.wrapping_add(1);

                        let pub_key = PublicKey::from_bytes(
                            &metadata_map.get(&METADATUM_2).unwrap().as_bytes().unwrap(),
                        )
                        .unwrap();

                        let stake_key = StakeKeyHex(PubKey(pub_key.as_bytes().try_into().unwrap()));
                        let rewards_address = Address::from_bytes(
                            metadata_map.get(&METADATUM_3).unwrap().as_bytes().unwrap(),
                        )
                        .unwrap();
                        let sig = Ed25519Signature::from_bytes(
                            signature_metadata_map
                                .get(&METADATUM_1)
                                .unwrap()
                                .as_bytes()
                                .unwrap(),
                        )
                        .unwrap();

                        SignedRegistration {
                            tx_id: TxId::from(tx_id),
                            slot: 0,
                            registration: Registration {
                                voting_key,
                                stake_key,
                                rewards_address: RewardsAddress(rewards_address.to_bytes().into()),
                                nonce: Nonce(
                                    u64::from_str(
                                        &metadata_map
                                            .get(&METADATUM_4)
                                            .unwrap()
                                            .as_int()
                                            .unwrap()
                                            .to_str(),
                                    )
                                    .unwrap(),
                                ),
                                voting_purpose: Some(VotingPurpose(0)),
                            },
                            signature: Signature {
                                inner: Sig(sig.to_bytes().try_into().unwrap()),
                            },
                            stake_key_hash: vec![0; 29],
                        }
                    })
                    .collect()
            })
            .fold(vec![], |mut acc, sub_vec: Vec<SignedRegistration>| {
                acc.extend(sub_vec);
                acc
            }))
    }

    fn stake_values(&self, stake_addrs: &[StakeKeyHash]) -> DashMap<StakeKeyHash, BigDecimal> {
        stake_addrs
            .iter()
            .map(|addr| {
                let big_num = self
                    .db_sync_instance
                    .stakes()
                    .get(&hex::encode(addr))
                    .unwrap_or(&BigNum::zero())
                    .to_string();
                (addr.clone(), BigDecimal::from_str(&big_num).unwrap())
            })
            .collect()
    }
}

impl From<InMemoryDbSync> for MockDbProvider {
    fn from(db_sync: InMemoryDbSync) -> Self {
        Self {
            db_sync_instance: db_sync,
        }
    }
}
*/