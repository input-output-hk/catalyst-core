use crate::certificate::{PoolId, PoolRegistration};
use crate::testing::data::address::AddressData;

use chain_crypto::{Curve25519_2HashDH, KeyPair, SumEd25519_12};

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
            id: id,
            vrf: vrf,
            kes: kes,
            pool_info: pool_info,
            reward_account: reward_account,
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
