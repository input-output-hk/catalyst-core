use crate::{
    jormungandr::explorer::{
        all_stake_pools::AllStakePoolsTipAllStakePools, data::stake_pool::StakePoolStakePool,
        verifiers::ExplorerVerifier,
    },
    utils::StakePool,
};
use chain_impl_mockchain::transaction::AccountIdentifier;
use chain_time::TimeOffsetSeconds;
use std::num::NonZeroU64;

impl ExplorerVerifier {
    pub fn assert_stake_pool(
        stake_pool: &StakePool,
        explorer_stake_pool: &StakePoolStakePool,
        retirement_time: core::option::Option<TimeOffsetSeconds>,
    ) {
        assert_eq!(explorer_stake_pool.id, stake_pool.id().to_string());
        assert_eq!(
            explorer_stake_pool.registration.management_threshold as u8,
            stake_pool.info().management_threshold()
        );

        assert_eq!(
            stake_pool.info().operators.len(),
            explorer_stake_pool.registration.operators.len()
        );

        assert_eq!(
            stake_pool.info().operators.len(),
            explorer_stake_pool.registration.operators.len()
        );
        assert!(explorer_stake_pool.registration.operators.iter().all(|operator| stake_pool
            .info()
            .operators
            .contains(&Self::decode_bech32_pk(operator))));

        assert_eq!(
            stake_pool.info().owners.len(),
            explorer_stake_pool.registration.owners.len()
        );

        assert!(explorer_stake_pool.registration.owners.iter().all(|owner| stake_pool
            .info()
            .owners
            .contains(&Self::decode_bech32_pk(owner))));

        if stake_pool.info().reward_account.is_some() {
            if let AccountIdentifier::Single(id) =
                stake_pool.info().reward_account.as_ref().unwrap()
            {
                assert_eq!(
                    id.to_string(),
                    explorer_stake_pool
                        .registration
                        .reward_account
                        .as_ref()
                        .unwrap()
                        .id
                );
            }
        }
        assert_eq!(
            u64::from(stake_pool.info().start_validity),
            explorer_stake_pool
                .registration
                .start_validity
                .parse::<u64>()
                .unwrap()
        );

        assert_eq!(
            stake_pool.info().rewards.ratio.numerator,
            explorer_stake_pool
                .registration
                .rewards
                .ratio
                .numerator
                .parse::<u64>()
                .unwrap()
        );
        assert_eq!(
            stake_pool.info().rewards.ratio.denominator,
            explorer_stake_pool
                .registration
                .rewards
                .ratio
                .denominator
                .parse::<NonZeroU64>()
                .unwrap()
        );

        if stake_pool.info().rewards.max_limit.is_some() {
            assert_eq!(
                stake_pool.info().rewards.max_limit.unwrap(),
                explorer_stake_pool
                    .registration
                    .rewards
                    .max_limit
                    .as_ref()
                    .unwrap()
                    .parse::<NonZeroU64>()
                    .unwrap()
            );
        }

        assert_eq!(
            explorer_stake_pool
                .registration
                .rewards
                .fixed
                .parse::<u64>()
                .unwrap(),
            stake_pool.info().rewards.fixed.0
        );

        if retirement_time.is_some() {
            assert!(explorer_stake_pool.retirement.is_some());
            assert_eq!(
                explorer_stake_pool.retirement.as_ref().unwrap().pool_id,
                stake_pool.info().to_id().to_string()
            );
            assert_eq!(
                u64::from(retirement_time.unwrap()),
                explorer_stake_pool
                    .retirement
                    .as_ref()
                    .unwrap()
                    .retirement_time
                    .parse::<u64>()
                    .unwrap()
            );
        }
    }

    pub fn assert_all_stake_pools(
        stake_pools: Vec<StakePool>,
        explorer_stake_pools: AllStakePoolsTipAllStakePools,
    ) {
        assert_eq!(stake_pools.len() as i64, explorer_stake_pools.total_count);
        let mut stake_pools_matching_count = 0;
        for stake_pool in &stake_pools {
            for explorer_stake_pool in &explorer_stake_pools.nodes {
                if explorer_stake_pool.id == stake_pool.id().to_string() {
                    stake_pools_matching_count += 1;
                    assert_eq!(
                        explorer_stake_pool.registration.management_threshold as u8,
                        stake_pool.info().management_threshold()
                    );

                    assert_eq!(
                        stake_pool.info().operators.len(),
                        explorer_stake_pool.registration.operators.len()
                    );

                    assert_eq!(
                        stake_pool.info().operators.len(),
                        explorer_stake_pool.registration.operators.len()
                    );

                    assert!(explorer_stake_pool.registration.operators.iter().all(|operator| stake_pool
                        .info()
                        .operators
                        .contains(&Self::decode_bech32_pk(operator))));

                    assert_eq!(
                        stake_pool.info().owners.len(),
                        explorer_stake_pool.registration.owners.len()
                    );
                    assert!(explorer_stake_pool.registration.owners.iter().all(|owner| stake_pool
                        .info()
                        .owners
                        .contains(&Self::decode_bech32_pk(owner))));


                    if stake_pool.info().reward_account.is_some() {
                        if let AccountIdentifier::Single(id) =
                            stake_pool.info().reward_account.as_ref().unwrap()
                        {
                            assert_eq!(
                                id.to_string(),
                                explorer_stake_pool
                                    .registration
                                    .reward_account
                                    .as_ref()
                                    .unwrap()
                                    .id
                            );
                        }
                    }
                    assert_eq!(
                        u64::from(stake_pool.info().start_validity),
                        explorer_stake_pool
                            .registration
                            .start_validity
                            .parse::<u64>()
                            .unwrap()
                    );

                    assert_eq!(
                        stake_pool.info().rewards.ratio.numerator,
                        explorer_stake_pool
                            .registration
                            .rewards
                            .ratio
                            .numerator
                            .parse::<u64>()
                            .unwrap()
                    );
                    assert_eq!(
                        stake_pool.info().rewards.ratio.denominator,
                        explorer_stake_pool
                            .registration
                            .rewards
                            .ratio
                            .denominator
                            .parse::<NonZeroU64>()
                            .unwrap()
                    );

                    if stake_pool.info().rewards.max_limit.is_some() {
                        assert_eq!(
                            stake_pool.info().rewards.max_limit.unwrap(),
                            explorer_stake_pool
                                .registration
                                .rewards
                                .max_limit
                                .as_ref()
                                .unwrap()
                                .parse::<NonZeroU64>()
                                .unwrap()
                        );
                    }

                    assert_eq!(
                        explorer_stake_pool
                            .registration
                            .rewards
                            .fixed
                            .parse::<u64>()
                            .unwrap(),
                        stake_pool.info().rewards.fixed.0
                    );
                }
            }
        }
        assert_eq!(stake_pools.len(), stake_pools_matching_count);
    }
}
