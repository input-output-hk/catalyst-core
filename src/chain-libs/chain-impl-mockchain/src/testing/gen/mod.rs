mod vote;

use crate::fragment::Fragment;
use crate::key::Hash;
use crate::{
    account::Identifier,
    certificate::PoolPermissions,
    config::ConfigParam,
    fragment::config::ConfigParams,
    header::VrfProof,
    key::BftLeaderId,
    ledger::Ledger,
    rewards::{Ratio, TaxType},
    setting::Settings,
    testing::{
        builders::StakePoolBuilder,
        data::{AddressData, LeaderPair, StakePool},
    },
    transaction::UnspecifiedAccountIdentifier,
    value::Value,
};
use chain_addr::Discrimination;
use chain_crypto::{vrf_evaluate_and_prove, Ed25519, KeyPair, PublicKey};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rand_core::RngCore;
use std::{iter, num::NonZeroU64};
pub use vote::VoteTestGen;

/// TestGen is a clone of abritrary architecture. There is a need to generate random struct,
/// which would be used just once (quickcheck run test method multiple times thus generating time-consuming test case).
/// This is needed for functional test approach rather than quickcheck approach
pub struct TestGen;

impl TestGen {
    pub fn hash() -> Hash {
        Hash::from_bytes(Self::bytes())
    }

    pub fn rand() -> ChaCha20Rng {
        ChaCha20Rng::from_seed(Self::bytes())
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

    pub fn unspecified_account_identifier() -> UnspecifiedAccountIdentifier {
        UnspecifiedAccountIdentifier::from_single_account(TestGen::identifier())
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

    pub fn ledger() -> Ledger {
        // TODO: Randomize some of the config paramaters below
        let leader_pair = TestGen::leader_pair();
        let header_id = TestGen::hash();
        let mut ie = ConfigParams::new();
        ie.push(ConfigParam::Discrimination(Discrimination::Test));
        ie.push(ConfigParam::AddBftLeader(leader_pair.leader_id));
        ie.push(ConfigParam::SlotDuration(10u8));
        ie.push(ConfigParam::SlotsPerEpoch(10u32));
        ie.push(ConfigParam::KesUpdateSpeed(3600));
        ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));

        Ledger::new(header_id, vec![&Fragment::Initial(ie)]).unwrap()
    }
}
