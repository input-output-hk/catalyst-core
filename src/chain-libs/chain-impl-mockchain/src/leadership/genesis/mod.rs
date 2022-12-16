mod vrfeval;

use crate::{
    certificate::PoolId,
    chaineval::PraosNonce,
    date::{BlockDate, Epoch},
    header::{Header, HeaderDesc, Proof},
    leadership::{Error, ErrorKind, Verification},
    setting::ActiveSlotsCoeff,
    stake::{PercentStake, PoolsState, Stake, StakeDistribution},
};
use chain_crypto::Verification as SigningVerification;
use chain_crypto::{RistrettoGroup2HashDh, SecretKey};
use thiserror::Error;
pub(crate) use vrfeval::witness_to_nonce;
use vrfeval::VrfEvaluator;
pub use vrfeval::{Threshold, VrfEvalFailure, Witness, WitnessOutput};

/// Genesis Praos leadership data for a specific epoch
pub struct LeadershipData {
    epoch_nonce: PraosNonce,
    nodes: PoolsState,
    distribution: StakeDistribution,
    // the epoch this leader selection is valid for
    epoch: Epoch,
    active_slots_coeff: ActiveSlotsCoeff,
}

#[derive(Debug, Error)]
enum GenesisError {
    #[error("Wrong epoch, expected epoch {expected} but received block at epoch {actual}")]
    InvalidEpoch { expected: Epoch, actual: Epoch },
    #[error("Total stake is null")]
    TotalStakeIsZero,
}

#[derive(Debug, Error)]
enum VrfError {
    #[error("Invalid Vrf Proof Structure in {bdesc} for poolId: {poolid}")]
    InvalidProofStructure { bdesc: HeaderDesc, poolid: String },
    #[error("Invalid Vrf Proof value in {bdesc}, poolId: {poolid}")]
    InvalidProofValue { bdesc: HeaderDesc, poolid: String },
    #[error("Vrf Threshold is not met in {bdesc}, poolId: {poolid}, pool_stake: {pool_stake}, total_stake: {total_stake}, value: {vrf_value}, threshold: {threshold}")]
    ThresholdInvalid {
        bdesc: HeaderDesc,
        poolid: String,
        pool_stake: Stake,
        total_stake: Stake,
        vrf_value: f64,
        threshold: f64,
    },
}

impl LeadershipData {
    pub fn new(
        epoch: Epoch,
        distribution: StakeDistribution,
        nodes: PoolsState,
        epoch_nonce: PraosNonce,
        active_slots_coeff: ActiveSlotsCoeff,
    ) -> Self {
        LeadershipData {
            epoch_nonce,
            nodes,
            distribution,
            epoch,
            active_slots_coeff,
        }
    }

    pub fn distribution(&self) -> &StakeDistribution {
        &self.distribution
    }

    pub fn nodes(&self) -> &PoolsState {
        &self.nodes
    }

    pub fn leader(
        &self,
        pool_id: &PoolId,
        vrf_key: &SecretKey<RistrettoGroup2HashDh>,
        date: BlockDate,
    ) -> Result<Option<Witness>, Error> {
        if date.epoch != self.epoch {
            return Err(Error::new_(
                ErrorKind::Failure,
                GenesisError::InvalidEpoch {
                    actual: date.epoch,
                    expected: self.epoch,
                },
            ));
        }

        let stake_snapshot = &self.distribution;

        match stake_snapshot.get_stake_for(pool_id) {
            None => Ok(None),
            Some(stake) => {
                // Calculate the total stake.
                let total_stake: Stake = stake_snapshot.total_stake();

                if total_stake == Stake::zero() {
                    return Err(Error::new_(
                        ErrorKind::Failure,
                        GenesisError::TotalStakeIsZero,
                    ));
                }

                let percent_stake = PercentStake {
                    stake,
                    total: total_stake,
                };

                let evaluator = VrfEvaluator {
                    stake: percent_stake,
                    nonce: &self.epoch_nonce,
                    slot_id: date.slot_id,
                    active_slots_coeff: self.active_slots_coeff,
                };
                Ok(evaluator.evaluate(vrf_key))
            }
        }
    }

    pub(crate) fn verify(&self, block_header: &Header) -> Verification {
        if block_header.block_date().epoch != self.epoch {
            return Verification::Failure(Error::new_(
                ErrorKind::Failure,
                GenesisError::InvalidEpoch {
                    expected: self.epoch,
                    actual: block_header.block_date().epoch,
                },
            ));
        }

        let stake_snapshot = &self.distribution;

        match block_header.proof() {
            Proof::GenesisPraos(ref genesis_praos_proof) => {
                let node_id = &genesis_praos_proof.node_id;
                match (
                    stake_snapshot.get_stake_for(node_id),
                    self.nodes.lookup_reg(node_id),
                ) {
                    (Some(stake), Some(pool_info)) => {
                        // Calculate the total stake.
                        let total_stake = stake_snapshot.total_stake();

                        let percent_stake = PercentStake::new(stake, total_stake);

                        let proof = match genesis_praos_proof.vrf_proof.to_vrf_proof() {
                            None => {
                                return Verification::Failure(Error::new_(
                                    ErrorKind::InvalidLeaderProof,
                                    VrfError::InvalidProofStructure {
                                        bdesc: block_header.description(),
                                        poolid: node_id.to_string(),
                                    },
                                ));
                            }
                            Some(p) => p,
                        };

                        let evaluator = VrfEvaluator {
                            stake: percent_stake,
                            nonce: &self.epoch_nonce,
                            slot_id: block_header.block_date().slot_id,
                            active_slots_coeff: self.active_slots_coeff,
                        };

                        match evaluator.verify(&pool_info.keys.vrf_public_key, &proof) {
                            // it would be the perfect place to keep the nonce ready to use for the context
                            // instead of ignoring but since the ledger.settings is not accessible,
                            // we recompute this later, expecting an already verified value.
                            Ok(_nonce) => (),
                            Err(VrfEvalFailure::ProofVerificationFailed) => {
                                return Verification::Failure(Error::new_(
                                    ErrorKind::InvalidLeaderProof,
                                    VrfError::InvalidProofValue {
                                        bdesc: block_header.description(),
                                        poolid: node_id.to_string(),
                                    },
                                ))
                            }
                            Err(VrfEvalFailure::ThresholdNotMet {
                                vrf_value,
                                stake_threshold,
                            }) => {
                                return Verification::Failure(Error::new_(
                                    ErrorKind::InvalidLeaderProof,
                                    VrfError::ThresholdInvalid {
                                        bdesc: block_header.description(),
                                        poolid: node_id.to_string(),
                                        pool_stake: stake,
                                        total_stake,
                                        vrf_value,
                                        threshold: stake_threshold,
                                    },
                                ));
                            }
                        };

                        let auth = block_header.as_auth_slice();
                        let valid = genesis_praos_proof
                            .kes_proof
                            .verify(&pool_info.keys.kes_public_key, auth);

                        if valid == SigningVerification::Failed {
                            Verification::Failure(Error::new(ErrorKind::InvalidLeaderSignature))
                        } else {
                            Verification::Success
                        }
                    }
                    (_, _) => Verification::Failure(Error::new(ErrorKind::InvalidBlockMessage)),
                }
            }
            _ => Verification::Failure(Error::new(ErrorKind::IncompatibleLeadershipMode)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::PoolId;
    use crate::chaintypes::HeaderId;
    use crate::ledger::Ledger;
    use crate::milli::Milli;
    use crate::stake::{PoolStakeDistribution, PoolStakeInformation};
    use crate::testing::{
        builders::{GenesisPraosBlockBuilder, StakePoolBuilder},
        ConfigBuilder, LedgerBuilder,
    };
    use crate::value::Value;
    use chain_core::property::ChainLength;
    use chain_crypto::{RistrettoGroup2HashDh, SecretKey};

    use std::collections::HashMap;

    fn make_pool(ledger: &mut Ledger) -> (PoolId, SecretKey<RistrettoGroup2HashDh>) {
        let stake_pool = StakePoolBuilder::new().build();
        ledger
            .delegation()
            .register_stake_pool(stake_pool.info())
            .expect("cannot add stake pool to ledger");
        (stake_pool.id(), stake_pool.vrf().private_key().clone())
    }

    #[derive(Clone, Debug)]
    pub struct LeaderElectionParameters {
        slots_per_epoch: u32,
        active_slots_coeff: f32,
        pools_count: usize,
        value: Stake,
    }

    impl LeaderElectionParameters {
        pub fn new() -> Self {
            // Those values are arbitrary. Generated by one of quickcheck test case
            // Converted it to 'standard' test case due to test case extended duration
            let pools_count = 5;
            let active_slots_coeff = 0.18;

            LeaderElectionParameters {
                slots_per_epoch: 1700,
                active_slots_coeff,
                pools_count,
                value: Stake::from_value(Value(100)),
            }
        }

        pub fn active_slots_coeff_as_milli(&self) -> Milli {
            Milli::from_millis((self.active_slots_coeff * 1000.0) as u64)
        }
    }

    type Pools = HashMap<PoolId, (SecretKey<RistrettoGroup2HashDh>, u64, Stake)>;

    fn make_leadership_with_pools(ledger: &Ledger, pools: &Pools) -> LeadershipData {
        let mut selection = LeadershipData::new(
            0,
            ledger.get_stake_distribution(),
            ledger.delegation.clone(),
            ledger.settings.consensus_nonce.clone(),
            ledger.settings.active_slots_coeff,
        );

        for (pool_id, (_, _, value)) in pools {
            update_stake_pool_total_value(&mut selection, pool_id, *value);
        }
        selection
    }

    fn update_stake_pool_total_value(
        selection: &mut LeadershipData,
        pool_id: &PoolId,
        value: Stake,
    ) {
        selection.distribution.to_pools.insert(
            pool_id.clone(),
            PoolStakeInformation {
                registration: None,
                stake: PoolStakeDistribution::test_new_with_total_value(value),
            },
        );
    }

    #[test]
    pub fn test_leader_with_invalid_pool_id() {
        // Arrange
        let leader_election_parameters = LeaderElectionParameters::new();

        let cb = ConfigBuilder::new()
            .with_slots_per_epoch(leader_election_parameters.slots_per_epoch)
            .with_active_slots_coeff(leader_election_parameters.active_slots_coeff_as_milli());

        let mut ledger = LedgerBuilder::from_config(cb)
            .build()
            .expect("cannot build test ledger")
            .ledger;

        let pools: HashMap<_, _> = std::iter::from_fn(|| {
            let (pool_id, pool_vrf_private_key) = make_pool(&mut ledger);
            Some((
                pool_id,
                (pool_vrf_private_key, 0, leader_election_parameters.value),
            ))
        })
        .take(leader_election_parameters.pools_count)
        .collect();

        let selection = make_leadership_with_pools(&ledger, &pools);

        let (invalid_pool_id, invalid_pool_vrf_private_key) = make_pool(&mut ledger);

        // Act
        let invalid_leader = selection.leader(
            &invalid_pool_id,
            &invalid_pool_vrf_private_key,
            ledger.date(),
        );

        // Assert
        assert!(invalid_leader.unwrap().is_none());
    }

    #[test]
    pub fn test_leader_election_is_consistent_with_stake_distribution() {
        let leader_election_parameters = LeaderElectionParameters::new();

        let cb = ConfigBuilder::new()
            .with_slots_per_epoch(leader_election_parameters.slots_per_epoch)
            .with_active_slots_coeff(leader_election_parameters.active_slots_coeff_as_milli());

        let mut ledger = LedgerBuilder::from_config(cb)
            .build()
            .expect("cannot build test ledger")
            .ledger;

        let mut pools = HashMap::<PoolId, (SecretKey<RistrettoGroup2HashDh>, u64, Stake)>::new();

        for _i in 0..leader_election_parameters.pools_count {
            let (pool_id, pool_vrf_private_key) = make_pool(&mut ledger);
            pools.insert(
                pool_id.clone(),
                (pool_vrf_private_key, 0, leader_election_parameters.value),
            );
        }

        let selection = make_leadership_with_pools(&ledger, &pools);

        let mut empty_slots = 0;
        let mut date = ledger.date();
        for _i in 0..leader_election_parameters.slots_per_epoch {
            let mut any_found = false;
            for (pool_id, (pool_vrf_private_key, times_selected, _)) in pools.iter_mut() {
                match selection
                    .leader(pool_id, pool_vrf_private_key, date)
                    .unwrap()
                {
                    None => {}
                    Some(_) => {
                        any_found = true;
                        *times_selected += 1;
                    }
                }
            }
            if !any_found {
                empty_slots += 1;
            }
            date = date.next(ledger.era());
        }

        println!("Calculating percentage of election per pool....");
        println!("parameters = {:?}", leader_election_parameters);
        println!("empty slots = {}", empty_slots);
        let total_election_count: u64 = pools.values().map(|y| y.1).sum();
        let ideal_election_count_per_pool: f32 =
            total_election_count as f32 / leader_election_parameters.pools_count as f32;
        let ideal_election_percentage = ideal_election_count_per_pool / total_election_count as f32;
        let grace_percentage: f32 = 0.08;
        println!(
            "ideal percentage: {:.2}, grace_percentage: {:.2}",
            ideal_election_percentage, grace_percentage
        );

        for (pool_id, (_pool_vrf_private_key, times_selected, stake)) in pools.iter_mut() {
            let pool_election_percentage = (*times_selected as f32) / (total_election_count as f32);
            println!(
                "pool id={}, stake={}, slots %={}",
                pool_id, stake, pool_election_percentage
            );

            assert!(
                (pool_election_percentage - ideal_election_percentage).abs() - grace_percentage
                    < 0.01,
                "Incorrect percentage {:.2} is out of correct range [{:.2} {:.2} ]",
                pool_election_percentage,
                ideal_election_percentage - grace_percentage,
                ideal_election_percentage + grace_percentage
            );
        }
    }

    #[test]
    #[ignore]
    pub fn test_phi() {
        let slots_per_epoch = 200_000;
        let active_slots_coeff = 0.1;
        let active_slots_coeff_as_milli = Milli::from_millis((active_slots_coeff * 1000.0) as u64);
        let cb = ConfigBuilder::new()
            .with_slots_per_epoch(slots_per_epoch)
            .with_active_slots_coeff(active_slots_coeff_as_milli);

        let mut ledger = LedgerBuilder::from_config(cb)
            .build()
            .expect("cannot build test ledger")
            .ledger;

        let mut pools = Pools::new();

        let (big_pool_id, big_pool_vrf_private_key) = make_pool(&mut ledger);
        pools.insert(
            big_pool_id.clone(),
            (big_pool_vrf_private_key, 0, Stake::from_value(Value(1000))),
        );

        for _i in 0..10 {
            let (small_pool_id, small_pool_vrf_private_key) = make_pool(&mut ledger);
            pools.insert(
                small_pool_id.clone(),
                (small_pool_vrf_private_key, 0, Stake::from_value(Value(100))),
            );
        }

        let selection = make_leadership_with_pools(&ledger, &pools);

        let mut date = ledger.date();

        let mut empty_slots = 0;

        let mut times_selected_small = 0;

        let nr_slots = slots_per_epoch;

        for _i in 0..nr_slots {
            let mut any_found = false;
            let mut any_small = false;
            for (pool_id, (pool_vrf_private_key, times_selected, value)) in pools.iter_mut() {
                match selection
                    .leader(pool_id, pool_vrf_private_key, date)
                    .unwrap()
                {
                    None => {}
                    Some(_witness) => {
                        any_found = true;
                        *times_selected += 1;
                        if *value == Stake::from_value(Value(100)) {
                            any_small = true;
                        }
                    }
                }
            }
            if !any_found {
                empty_slots += 1;
            }
            if any_small {
                times_selected_small += 1;
            }
            date = date.next(ledger.era());
        }

        for (pool_id, (_pool_vrf_private_key, times_selected, stake)) in pools.iter_mut() {
            println!(
                "pool id={} stake={} slots={}",
                pool_id, stake, times_selected
            );
        }
        println!("empty slots = {}", empty_slots);
        println!("small stake slots = {}", times_selected_small);
        let times_selected_big = pools[&big_pool_id].1;
        println!("big stake slots = {}", times_selected_big);

        // Check that we got approximately the correct number of active slots.
        assert!(empty_slots > (nr_slots as f64 * (1.0 - active_slots_coeff - 0.01)) as u32);
        assert!(empty_slots < (nr_slots as f64 * (1.0 - active_slots_coeff + 0.01)) as u32);

        // Check that splitting a stake doesn't have a big effect on
        // the chance of becoming slot leader.
        assert!((times_selected_big as f64 / times_selected_small as f64) > 0.98);
        assert!((times_selected_big as f64 / times_selected_small as f64) < 1.02);
    }

    #[test]
    pub fn leadership_leader_different_epoch() {
        let selection_epoch = 0;
        let date = BlockDate {
            epoch: 1u32,
            slot_id: 0u32,
        };
        let mut ledger = LedgerBuilder::from_config(ConfigBuilder::new())
            .build()
            .expect("cannot build test ledger")
            .ledger;

        let stake_pool = StakePoolBuilder::new().build();
        *ledger.delegation_mut() = ledger
            .delegation()
            .register_stake_pool(stake_pool.info())
            .expect("cannot register stake pool");
        let mut selection = LeadershipData::new(
            selection_epoch,
            ledger.get_stake_distribution(),
            ledger.delegation.clone(),
            ledger.settings.consensus_nonce,
            ledger.settings.active_slots_coeff,
        );
        update_stake_pool_total_value(
            &mut selection,
            &stake_pool.id(),
            Stake::from_value(Value(100)),
        );

        assert!(selection
            .leader(&stake_pool.id(), stake_pool.vrf().private_key(), date)
            .is_err());
    }

    #[test]
    pub fn leadership_leader_no_stake() {
        let date = BlockDate::first();
        let mut ledger = LedgerBuilder::from_config(ConfigBuilder::new())
            .build()
            .expect("cannot build test ledger")
            .ledger;

        let stake_pool = StakePoolBuilder::new().build();
        *ledger.delegation_mut() = ledger
            .delegation()
            .register_stake_pool(stake_pool.info())
            .expect("cannot register stake pool");
        let selection = LeadershipData::new(
            date.epoch,
            ledger.get_stake_distribution(),
            ledger.delegation.clone(),
            ledger.settings.consensus_nonce,
            ledger.settings.active_slots_coeff,
        );

        assert!(selection
            .leader(&stake_pool.id(), stake_pool.vrf().private_key(), date)
            .is_err());
    }

    #[test]
    pub fn leadership_leader_zero_stake() {
        let date = BlockDate::first();
        let mut ledger = LedgerBuilder::from_config(ConfigBuilder::new())
            .build()
            .expect("cannot build test ledger")
            .ledger;

        let stake_pool = StakePoolBuilder::new().build();
        *ledger.delegation_mut() = ledger
            .delegation()
            .register_stake_pool(stake_pool.info())
            .expect("cannot register stake pool");
        let mut selection = LeadershipData::new(
            date.epoch,
            ledger.get_stake_distribution(),
            ledger.delegation.clone(),
            ledger.settings.consensus_nonce,
            ledger.settings.active_slots_coeff,
        );
        update_stake_pool_total_value(&mut selection, &stake_pool.id(), Stake::zero());

        assert!(selection
            .leader(&stake_pool.id(), stake_pool.vrf().private_key(), date)
            .is_err());
    }

    use crate::fragment::Contents;
    use crate::header::{BlockVersion, HeaderBuilderNew};

    #[test]
    pub fn leadership_verify_different_epoch() {
        let date = BlockDate {
            epoch: 1,
            slot_id: 0,
        };
        let testledger = LedgerBuilder::from_config(ConfigBuilder::new())
            .build()
            .expect("cannot build test ledger");
        let mut ledger = testledger.ledger;

        let stake_pool = StakePoolBuilder::new().build();
        *ledger.delegation_mut() = ledger
            .delegation()
            .register_stake_pool(stake_pool.info())
            .expect("cannot register stake pool");
        let selection = LeadershipData::new(
            0,
            ledger.get_stake_distribution(),
            ledger.delegation.clone(),
            ledger.settings.consensus_nonce.clone(),
            ledger.settings.active_slots_coeff,
        );

        let block = GenesisPraosBlockBuilder::new()
            .with_date(date)
            .with_chain_length(ledger.chain_length())
            .with_parent_id(testledger.block0_hash)
            .build(&stake_pool, ledger.era());

        assert!(selection.verify(block.header()).failure());
    }

    #[test]
    pub fn leadership_verify_different_proof() {
        let date = BlockDate {
            epoch: 1,
            slot_id: 0,
        };
        let mut ledger = LedgerBuilder::from_config(ConfigBuilder::new())
            .build()
            .expect("cannot build test ledger")
            .ledger;

        let stake_pool = StakePoolBuilder::new().build();
        *ledger.delegation_mut() = ledger
            .delegation()
            .register_stake_pool(stake_pool.info())
            .expect("cannot register stake pool");
        let selection = LeadershipData::new(
            date.epoch,
            ledger.get_stake_distribution(),
            ledger.delegation.clone(),
            ledger.settings.consensus_nonce.clone(),
            ledger.settings.active_slots_coeff,
        );
        let rng = rand_core::OsRng;
        let sk = &SecretKey::generate(rng);
        let header = HeaderBuilderNew::new(BlockVersion::Ed25519Signed, &Contents::empty())
            .set_parent(&HeaderId::zero_hash(), ledger.chain_length().increase())
            .set_date(date)
            .into_bft_builder()
            .unwrap()
            .sign_using(sk)
            .generalize();

        assert!(selection.verify(&header).failure());
    }

    #[test]
    pub fn leadership_verify_no_stake() {
        let date = BlockDate::first();
        let testledger = LedgerBuilder::from_config(ConfigBuilder::new())
            .build()
            .expect("cannot build test ledger");
        let mut ledger = testledger.ledger;

        let stake_pool = StakePoolBuilder::new().build();
        *ledger.delegation_mut() = ledger
            .delegation()
            .register_stake_pool(stake_pool.info())
            .expect("cannot register stake pool");
        let selection = LeadershipData::new(
            date.epoch,
            ledger.get_stake_distribution(),
            ledger.delegation.clone(),
            ledger.settings.consensus_nonce.clone(),
            ledger.settings.active_slots_coeff,
        );

        let block = GenesisPraosBlockBuilder::new()
            .with_date(date)
            .with_chain_length(ledger.chain_length())
            .with_parent_id(testledger.block0_hash)
            .build(&stake_pool, ledger.era());
        assert!(selection.verify(block.header()).failure());
    }

    #[test]
    pub fn leadership_verify_zero_stake() {
        let date = BlockDate::first();
        let testledger = LedgerBuilder::from_config(ConfigBuilder::new())
            .build()
            .expect("cannot build test ledger");
        let mut ledger = testledger.ledger;

        let stake_pool = StakePoolBuilder::new().build();
        *ledger.delegation_mut() = ledger
            .delegation()
            .register_stake_pool(stake_pool.info())
            .expect("cannot register stake pool");
        let mut selection = LeadershipData::new(
            date.epoch,
            ledger.get_stake_distribution(),
            ledger.delegation.clone(),
            ledger.settings.consensus_nonce.clone(),
            ledger.settings.active_slots_coeff,
        );
        update_stake_pool_total_value(&mut selection, &stake_pool.id(), Stake::zero());

        let block = GenesisPraosBlockBuilder::new()
            .with_date(date)
            .with_chain_length(ledger.chain_length())
            .with_parent_id(testledger.block0_hash)
            .build(&stake_pool, ledger.era());

        assert!(selection.verify(block.header()).failure());
    }

    #[test]
    pub fn leadership_verify_non_existing_pool() {
        let date = BlockDate::first();
        let testledger = LedgerBuilder::from_config(ConfigBuilder::new())
            .build()
            .expect("cannot build test ledger");
        let ledger = testledger.ledger;

        let stake_pool = StakePoolBuilder::new().build();
        let selection = LeadershipData::new(
            date.epoch,
            ledger.get_stake_distribution(),
            ledger.delegation.clone(),
            ledger.settings.consensus_nonce.clone(),
            ledger.settings.active_slots_coeff,
        );

        let block = GenesisPraosBlockBuilder::new()
            .with_date(date)
            .with_chain_length(ledger.chain_length())
            .with_parent_id(testledger.block0_hash)
            .build(&stake_pool, ledger.era());

        assert!(selection.verify(block.header()).failure());
    }

    #[test]
    pub fn leadership_not_in_the_current_epoch() {
        let date = BlockDate {
            epoch: 2,
            slot_id: 0,
        };
        let mut ledger = LedgerBuilder::from_config(ConfigBuilder::new())
            .build()
            .expect("cannot build test ledger")
            .ledger;

        let stake_pool = StakePoolBuilder::new().build();
        *ledger.delegation_mut() = ledger
            .delegation()
            .register_stake_pool(stake_pool.info())
            .expect("cannot register stake pool");
        let selection = LeadershipData::new(
            date.epoch,
            ledger.get_stake_distribution(),
            ledger.delegation.clone(),
            ledger.settings.consensus_nonce.clone(),
            ledger.settings.active_slots_coeff,
        );
        let rng = rand_core::OsRng;
        let sk = &SecretKey::generate(rng);
        let header = HeaderBuilderNew::new(BlockVersion::Ed25519Signed, &Contents::empty())
            .set_parent(&HeaderId::zero_hash(), ledger.chain_length().next())
            .set_date(date)
            .into_bft_builder()
            .unwrap()
            .sign_using(sk)
            .generalize();

        assert!(selection.verify(&header).failure());
    }
}
