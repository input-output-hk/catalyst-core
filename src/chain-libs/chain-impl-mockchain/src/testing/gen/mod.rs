mod vote;

use crate::fragment::Contents;
use crate::fragment::Fragment;
use crate::header::BlockDate;
use crate::header::ChainLength;
use crate::header::Header;
use crate::header::HeaderBuilderNew;
use crate::header::{BlockVersion, HeaderId};
use crate::key::Hash;
use crate::ledger::LedgerStaticParameters;
use crate::{
    account::Identifier,
    certificate::{MintToken, PoolPermissions},
    config::ConfigParam,
    fragment::config::ConfigParams,
    header::VrfProof,
    ledger::Ledger,
    rewards::{Ratio, TaxType},
    setting::Settings,
    testing::{
        builders::StakePoolBuilder,
        data::{AddressData, LeaderPair, StakePool},
    },
    tokens::{
        identifier::TokenIdentifier,
        name::{TokenName, TOKEN_NAME_MAX_SIZE},
        policy_hash::POLICY_HASH_SIZE,
    },
    transaction::UnspecifiedAccountIdentifier,
    value::Value,
};
use chain_addr::Discrimination;
use chain_crypto::SecretKey;
use chain_crypto::{vrf_evaluate_and_prove, Ed25519, KeyPair, PublicKey};
use chain_time::{Epoch as TimeEpoch, SlotDuration, TimeEra, TimeFrame, Timeline};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rand_core::RngCore;
use std::time::Duration;
use std::time::SystemTime;
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

    pub fn parent_id() -> HeaderId {
        Self::hash()
    }

    pub fn leader_pair() -> LeaderPair {
        let leader_key = AddressData::generate_key_pair::<Ed25519>()
            .private_key()
            .clone();
        LeaderPair::new(leader_key)
    }

    pub fn secret_key() -> SecretKey<Ed25519> {
        AddressData::generate_key_pair::<Ed25519>()
            .private_key()
            .clone()
    }

    pub fn secret_keys() -> impl Iterator<Item = SecretKey<Ed25519>> {
        iter::from_fn(|| {
            Some(
                AddressData::generate_key_pair::<Ed25519>()
                    .private_key()
                    .clone(),
            )
        })
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

    pub fn chain_length() -> ChainLength {
        ChainLength(Self::rand().next_u32())
    }

    pub fn genesis_praos_header() -> Header {
        let stake_pool = TestGen::stake_pool();

        HeaderBuilderNew::new(BlockVersion::KesVrfproof, &Contents::empty())
            .set_parent(&TestGen::hash(), TestGen::chain_length())
            .set_date(BlockDate {
                epoch: 0,
                slot_id: 1,
            })
            .into_genesis_praos_builder()
            .unwrap()
            .set_consensus_data(&stake_pool.id(), &TestGen::vrf_proof(&stake_pool))
            .sign_using(stake_pool.kes().private_key())
            .generalize()
    }

    pub fn bft_header() -> Header {
        let stake_pool = TestGen::stake_pool();

        HeaderBuilderNew::new(BlockVersion::KesVrfproof, &Contents::empty())
            .set_parent(&TestGen::hash(), TestGen::chain_length())
            .set_date(BlockDate {
                epoch: 0,
                slot_id: 1,
            })
            .into_genesis_praos_builder()
            .unwrap()
            .set_consensus_data(&stake_pool.id(), &TestGen::vrf_proof(&stake_pool))
            .sign_using(stake_pool.kes().private_key())
            .generalize()
    }

    pub fn ledger() -> Ledger {
        // TODO: Randomize some of the config paramaters below
        let leader_pair = TestGen::leader_pair();
        let header_id = TestGen::hash();
        let mut ie = ConfigParams::new();
        ie.push(ConfigParam::Discrimination(Discrimination::Test));
        ie.push(ConfigParam::AddBftLeader(leader_pair.id()));
        ie.push(ConfigParam::SlotDuration(10u8));
        ie.push(ConfigParam::SlotsPerEpoch(10u32));
        ie.push(ConfigParam::KesUpdateSpeed(3600));
        ie.push(ConfigParam::Block0Date(crate::config::Block0Date(0)));

        Ledger::new(header_id, vec![&Fragment::Initial(ie)]).unwrap()
    }

    pub fn static_parameters() -> LedgerStaticParameters {
        LedgerStaticParameters {
            block0_initial_hash: Self::hash(),
            block0_start_time: crate::config::Block0Date(0),
            discrimination: Discrimination::Test,
            kes_update_speed: 0,
        }
    }

    pub fn time_era() -> TimeEra {
        let now = SystemTime::now();
        let t0 = Timeline::new(now);
        let f0 = SlotDuration::from_secs(5);
        let tf0 = TimeFrame::new(t0, f0);
        let t1 = now + Duration::from_secs(10);
        let slot1 = tf0.slot_at(&t1).unwrap();
        TimeEra::new(slot1, TimeEpoch(2), 4)
    }

    pub fn token_id() -> TokenIdentifier {
        let mut rng = rand_core::OsRng;

        let mut policy_hash: [u8; POLICY_HASH_SIZE] = [0; POLICY_HASH_SIZE];
        rng.fill_bytes(&mut policy_hash);

        TokenIdentifier {
            policy_hash: TryFrom::try_from(policy_hash).unwrap(),
            token_name: Self::token_name(),
        }
    }

    pub fn token_name() -> TokenName {
        let mut rng = rand_core::OsRng;
        let mut token_name: [u8; TOKEN_NAME_MAX_SIZE] = [0; TOKEN_NAME_MAX_SIZE];
        rng.fill_bytes(&mut token_name);
        TryFrom::try_from(token_name.to_vec()).unwrap()
    }

    pub fn mint_token_for_wallet(id: Identifier) -> MintToken {
        MintToken {
            name: TestGen::token_name(),
            policy: Default::default(),
            to: id,
            value: Value(1),
        }
    }
}
