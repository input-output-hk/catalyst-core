pub mod arbitrary;
pub mod builders;
pub mod data;
pub mod e2e;
pub mod ledger;
pub mod scenario;
pub mod verifiers;
pub use arbitrary::*;
pub use builders::*;
pub use data::KeysDb;
pub use ledger::{ConfigBuilder, LedgerBuilder, TestLedger, UtxoDb};

pub use chain_crypto::testing::TestCryptoGen;

use crate::key::Hash;
use crate::{
    account::Identifier,
    certificate::PoolPermissions,
    config::ConfigParam,
    fragment::config::ConfigParams,
    header::VrfProof,
    key::BftLeaderId,
    rewards::{Ratio, TaxType},
    setting::Settings,
    testing::data::{AddressData, LeaderPair, StakePool},
    value::Value,
};

use chain_crypto::{vrf_evaluate_and_prove, Ed25519, KeyPair, PublicKey};
use rand_core::RngCore;
use std::{iter, num::NonZeroU64};

pub struct TestGen;

impl TestGen {
    pub fn hash() -> Hash {
        Hash::from_bytes(Self::bytes())
    }

    pub fn bytes() -> [u8; 32] {
        let mut random_bytes: [u8; 32] = [0; 32];
        let mut rng = rand_core::OsRng;
        rng.fill_bytes(&mut random_bytes);
        random_bytes
    }

    pub fn identifier() -> Identifier {
        let kp: KeyPair<Ed25519> = AddressData::generate_key_pair::<Ed25519>();
        Identifier::from(kp.into_keys().1)
    }

    pub fn public_key() -> PublicKey<Ed25519> {
        AddressData::generate_key_pair::<Ed25519>()
            .public_key()
            .clone()
    }

    pub fn leader_pair() -> LeaderPair {
        let leader_key = AddressData::generate_key_pair::<Ed25519>()
            .private_key()
            .clone();
        let leader_id = BftLeaderId(
            AddressData::generate_key_pair::<Ed25519>()
                .public_key()
                .clone(),
        );
        LeaderPair::new(leader_id, leader_key)
    }

    pub fn leaders_pairs() -> impl Iterator<Item = LeaderPair> {
        iter::from_fn(|| Some(TestGen::leader_pair()))
    }

    pub fn settings(leaders: Vec<LeaderPair>) -> Settings {
        let settings = Settings::new();
        let mut config_params = ConfigParams::new();
        for leader_id in leaders.iter().cloned().map(|x| x.id()) {
            config_params.push(ConfigParam::AddBftLeader(leader_id));
        }
        settings.apply(&config_params).unwrap()
    }

    pub fn vrf_proof(stake_pool: &StakePool) -> VrfProof {
        let mut rng = rand_core::OsRng;
        vrf_evaluate_and_prove(stake_pool.vrf().private_key(), &TestGen::bytes(), &mut rng).into()
    }

    pub fn stake_pool() -> StakePool {
        StakePoolBuilder::new()
            .with_owners(vec![TestGen::public_key()])
            .with_operators(vec![TestGen::public_key()])
            .with_pool_permissions(PoolPermissions::new(1u8))
            .with_reward_account(false)
            .with_tax_type(TaxType {
                fixed: Value(0),
                ratio: Ratio {
                    numerator: 1,
                    denominator: NonZeroU64::new(2).unwrap(),
                },
                max_limit: Some(NonZeroU64::new(100).unwrap()),
            })
            .build()
    }
}
