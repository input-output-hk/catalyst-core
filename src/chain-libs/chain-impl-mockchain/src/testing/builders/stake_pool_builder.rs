use crate::{
    certificate::{PoolPermissions, PoolRegistration},
    leadership::genesis::GenesisPraosLeader,
    rewards::{Ratio, TaxType},
    testing::data::{AddressData, StakePool},
    transaction::AccountIdentifier,
    value::Value,
};
use chain_addr::Discrimination;
use chain_crypto::{Curve25519_2HashDH, Ed25519, KeyPair, PublicKey, SumEd25519_12};
use chain_time::DurationSeconds;
use std::num::NonZeroU64;

pub struct StakePoolBuilder {
    owners: Vec<PublicKey<Ed25519>>,
    operators: Vec<PublicKey<Ed25519>>,
    pool_permissions: Option<PoolPermissions>,
    reward_account: bool,
    tax_type: TaxType,
    alias: String,
}

impl StakePoolBuilder {
    pub fn new() -> Self {
        StakePoolBuilder {
            owners: Vec::new(),
            operators: Vec::new(),
            alias: "".to_owned(),
            pool_permissions: None,
            reward_account: false,
            tax_type: TaxType {
                fixed: Value(1),
                ratio: Ratio {
                    numerator: 0u64,
                    denominator: NonZeroU64::new(1u64).unwrap(),
                },
                max_limit: None,
            },
        }
    }

    pub fn with_owners(&mut self, owners: Vec<PublicKey<Ed25519>>) -> &mut Self {
        self.owners.extend(owners);
        self
    }

    pub fn with_alias(&mut self, alias: &str) -> &mut Self {
        self.alias = alias.to_owned();
        self
    }

    pub fn with_operators(&mut self, operators: Vec<PublicKey<Ed25519>>) -> &mut Self {
        self.operators.extend(operators);
        self
    }

    pub fn with_pool_permissions(&mut self, permissions: PoolPermissions) -> &mut Self {
        self.pool_permissions = Some(permissions);
        self
    }

    pub fn with_reward_account(&mut self, reward_account: bool) -> &mut Self {
        self.reward_account = reward_account;
        self
    }

    pub fn with_tax_type(&mut self, tax_type: TaxType) -> &mut Self {
        self.tax_type = tax_type;
        self
    }

    pub fn build(&self) -> StakePool {
        let mut rng = rand_os::OsRng::new().unwrap();

        let pool_vrf: KeyPair<Curve25519_2HashDH> = KeyPair::generate(&mut rng);
        let pool_kes: KeyPair<SumEd25519_12> = KeyPair::generate(&mut rng);

        let permissions = match &self.pool_permissions {
            Some(pool_permissions) => pool_permissions.clone(),
            None => PoolPermissions::new(std::cmp::max(self.owners.len() as u8 / 2, 1)),
        };

        let (reward_account, reward_identifier) = match self.reward_account {
            true => {
                let account = AddressData::account(Discrimination::Test);
                let transaction_account = AccountIdentifier::Single(account.to_id());
                (Some(account), Some(transaction_account))
            }
            false => (None, None),
        };

        let pool_info = PoolRegistration {
            serial: 1234,
            owners: self.owners.iter().cloned().collect(),
            operators: self.operators.iter().cloned().collect(),
            start_validity: DurationSeconds::from(0).into(),
            permissions: permissions,
            rewards: self.tax_type.clone(),
            reward_account: reward_identifier,
            keys: GenesisPraosLeader {
                vrf_public_key: pool_vrf.public_key().clone(),
                kes_public_key: pool_kes.public_key().clone(),
            },
        };
        StakePool::new(
            &self.alias,
            pool_info.to_id(),
            pool_vrf,
            pool_kes,
            pool_info,
            reward_account,
        )
    }
}
