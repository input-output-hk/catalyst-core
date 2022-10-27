use crate::{
    jormungandr::explorer::{data::stake_pool::StakePoolStakePool, verifiers::ExplorerVerifier},
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
    )
    /*-> Result<(), VerifierError>*/
    {
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
        let mut operators_matching = 0;
        for operator in &explorer_stake_pool.registration.operators {
            if stake_pool
                .info()
                .operators
                .contains(&Self::decode_bech32_pk(&operator))
            {
                operators_matching += 1;
            }
        }
        assert_eq!(stake_pool.info().operators.len(), operators_matching);

        assert_eq!(
            stake_pool.info().owners.len(),
            explorer_stake_pool.registration.owners.len()
        );
        let mut owners_matching = 0;
        for owner in &explorer_stake_pool.registration.owners {
            if stake_pool
                .info()
                .owners
                .contains(&Self::decode_bech32_pk(&owner))
            {
                owners_matching += 1;
            }
        }
        assert_eq!(stake_pool.info().owners.len(), owners_matching);

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
}
