use crate::{
    certificate::{PoolId, PoolRegistration},
    testing::{builders::StakePoolBuilder, data::address::AddressData, TestGen},
};

use chain_crypto::{Curve25519_2HashDH, Ed25519, KeyPair, PublicKey, SumEd25519_12};
use quickcheck::{Arbitrary, Gen};
use std::iter;

#[derive(Clone, Debug)]
pub struct StakePool {
    alias: String,
    id: PoolId,
    vrf: KeyPair<Curve25519_2HashDH>,
    kes: KeyPair<SumEd25519_12>,
    pool_info: PoolRegistration,
    reward_account: Option<AddressData>,
}

impl StakePool {
    pub fn new(
        alias: &str,
        id: PoolId,
        vrf: KeyPair<Curve25519_2HashDH>,
        kes: KeyPair<SumEd25519_12>,
        pool_info: PoolRegistration,
        reward_account: Option<AddressData>,
    ) -> Self {
        StakePool {
            alias: alias.to_owned(),
            id,
            vrf,
            kes,
            pool_info,
            reward_account,
        }
    }

    pub fn id(&self) -> PoolId {
        self.id.clone()
    }

    pub fn vrf(&self) -> KeyPair<Curve25519_2HashDH> {
        self.vrf.clone()
    }

    pub fn kes(&self) -> KeyPair<SumEd25519_12> {
        self.kes.clone()
    }

    pub fn info_mut(&mut self) -> &mut PoolRegistration {
        &mut self.pool_info
    }

    pub fn info(&self) -> PoolRegistration {
        self.pool_info.clone()
    }

    pub fn alias(&self) -> String {
        self.alias.clone()
    }

    pub fn reward_account(&self) -> Option<&AddressData> {
        self.reward_account.as_ref()
    }
}

impl Arbitrary for StakePool {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        let owners_count = std::cmp::max(usize::arbitrary(gen) % 8, 1);
        let operators_count = usize::arbitrary(gen) % 4;
        let owners: Vec<PublicKey<Ed25519>> = iter::from_fn(|| Some(TestGen::public_key()))
            .take(owners_count)
            .collect();
        let operators: Vec<PublicKey<Ed25519>> = iter::from_fn(|| Some(TestGen::public_key()))
            .take(operators_count)
            .collect();

        StakePoolBuilder::new()
            .with_owners(owners)
            .with_operators(operators)
            .with_pool_permissions(Arbitrary::arbitrary(gen))
            .with_reward_account(Arbitrary::arbitrary(gen))
            .with_tax_type(Arbitrary::arbitrary(gen))
            .build()
    }
}
