use quickcheck::{Arbitrary, Gen};

use crate::{
    certificate::PoolPermissions,
    fragment::Fragment,
    testing::{
        arbitrary::{utils, WalletCollection},
        builders::{
            create_initial_stake_pool_delegation, create_initial_stake_pool_registration,
            StakePoolBuilder,
        },
        data::StakePool,
        ledger::{ConfigBuilder, LedgerBuilder, UtxoDeclaration},
    },
};
use chain_addr::Discrimination;

impl Arbitrary for LedgerBuilder {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let utxos = WalletCollection::arbitrary(g);
        let utxo_declarations: Vec<UtxoDeclaration> =
            utxos.0.iter().map(|utxo| utxo.make_output()).collect();

        let faucets = WalletCollection::arbitrary(g);
        let splits = utils::split_vec(&faucets.0, g, 3);
        let stake_pools_owners = splits.get(0).unwrap();
        let stake_pools: Vec<StakePool> = stake_pools_owners
            .iter()
            .cloned()
            .map(|faucet| {
                StakePoolBuilder::new()
                    .with_owners(vec![faucet.public_key()])
                    .with_pool_permissions(PoolPermissions::new(1))
                    .with_reward_account(Arbitrary::arbitrary(g))
                    .with_tax_type(Arbitrary::arbitrary(g))
                    .build()
            })
            .collect();

        let mut fragments: Vec<Fragment> = Vec::new();
        let registration_certs: Vec<Fragment> = stake_pools_owners
            .iter()
            .cloned()
            .zip(stake_pools.iter())
            .map(|(owner, stake_pool)| create_initial_stake_pool_registration(stake_pool, &[owner]))
            .collect();
        fragments.extend(registration_certs.iter().cloned());

        let owner_delegation_certs: Vec<Fragment> = stake_pools_owners
            .iter()
            .zip(stake_pools.iter())
            .map(|(owner, stake_pool)| create_initial_stake_pool_delegation(stake_pool, owner))
            .collect();
        fragments.extend(owner_delegation_certs.iter().cloned());

        let stake_pools_delegators = splits.get(1).unwrap();
        let mut stake_pools_cycle = stake_pools.iter().cycle();
        for wallet in stake_pools_delegators {
            fragments.push(create_initial_stake_pool_delegation(
                &stake_pools_cycle.next().unwrap(),
                wallet,
            ));
        }

        let config_builder = ConfigBuilder::arbitrary(g).with_discrimination(Discrimination::Test);

        LedgerBuilder::from_config(config_builder)
            .faucets_wallets(faucets.0.iter().collect())
            .certs(&fragments)
            .utxos(&utxo_declarations)
    }
}
