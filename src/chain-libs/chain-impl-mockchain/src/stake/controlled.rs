use crate::{
    account::{self, Identifier},
    rewards::Ratio,
    stake::Stake,
    utxo,
};
use chain_addr::{Address, Kind};
use imhamt::Hamt;
use std::{collections::hash_map::DefaultHasher, fmt, num::NonZeroU64};

#[derive(Default, Clone, Eq, PartialEq)]
pub struct StakeControl {
    assigned: Stake,
    unassigned: Stake,

    control: Hamt<DefaultHasher, Identifier, Stake>,
}

impl StakeControl {
    pub fn new() -> Self {
        Self::default()
    }

    fn update_accounts(&self, accounts: &account::Ledger) -> Self {
        accounts
            .iter()
            .fold(self.clone(), |sc, (identifier, account)| {
                sc.add_to(identifier.clone(), Stake::from_value(account.value()))
            })
    }

    fn update_utxos(&self, utxos: &utxo::Ledger<Address>) -> Self {
        utxos.values().fold(self.clone(), |sc, output| {
            let stake = Stake::from_value(output.value);

            // We're only interested in "group" addresses
            // (i.e. containing a spending key and a stake key).
            match output.address.kind() {
                Kind::Account(_) | Kind::Multisig(_) => {
                    // single or multisig account are not present in utxos
                    panic!("internal error: accounts in utxo")
                }
                Kind::Script(_) => {
                    // scripts are not present in utxo
                    panic!("internal error: script in utxo")
                }
                Kind::Group(_spending_key, account_key) => {
                    let identifier = account_key.clone().into();
                    sc.add_to(identifier, stake)
                }
                Kind::Single(_) => sc.add_unassigned(stake),
            }
        })
    }

    pub fn new_with(accounts: &account::Ledger, utxos: &utxo::Ledger<Address>) -> Self {
        Self::new().update_accounts(accounts).update_utxos(utxos)
    }

    pub fn total(&self) -> Stake {
        self.assigned + self.unassigned
    }

    pub fn assigned(&self) -> Stake {
        self.assigned
    }

    pub fn unassigned(&self) -> Stake {
        self.unassigned
    }

    /// get the total stake controlled by the given account
    pub fn by(&self, identifier: &Identifier) -> Option<Stake> {
        self.control.lookup(identifier).copied()
    }

    /// get the ratio controlled by the given account
    ///
    /// the ratio is based on the total assigned stake, stake that is
    /// not controlled (that is in UTxO without account keys) are not
    /// part of the equation.
    ///
    pub fn ratio_by(&self, identifier: &Identifier) -> Ratio {
        if let Some(stake) = self.by(identifier) {
            Ratio {
                numerator: stake.0,
                denominator: unsafe {
                    // the assigned cannot be `0` because there must
                    // be at least the account's stake which is non
                    // nul
                    NonZeroU64::new_unchecked(self.assigned().0)
                },
            }
        } else {
            Ratio::zero()
        }
    }

    #[must_use = "internal state is not modified"]
    pub fn add_unassigned(&self, stake: Stake) -> Self {
        Self {
            assigned: self.assigned,
            unassigned: self.unassigned.wrapping_add(stake),
            control: self.control.clone(),
        }
    }

    #[must_use = "internal state is not modified"]
    pub fn remove_unassigned(&self, stake: Stake) -> Self {
        Self {
            assigned: self.assigned,
            unassigned: self.unassigned.wrapping_sub(stake),
            control: self.control.clone(),
        }
    }

    /// add the given amount of stake to the given identifier
    ///
    /// also update the total stake
    #[must_use = "internal state is not modified"]
    pub fn add_to(&self, identifier: Identifier, stake: Stake) -> Self {
        let control = self
            .control
            .insert_or_update_simple(identifier, stake, |v: &Stake| v.checked_add(stake));

        Self {
            control,
            assigned: self.assigned.wrapping_add(stake),
            unassigned: self.unassigned,
        }
    }

    /// add the given amount of stake to the given identifier
    ///
    /// also update the total stake
    #[must_use = "internal state is not modified"]
    pub fn remove_from(&self, identifier: Identifier, stake: Stake) -> Self {
        let control = self.control.update(&identifier, |v| {
            Result::<Option<Stake>, String>::Ok(v.checked_sub(stake))
        });

        let control = match control {
            Ok(updated) => updated,
            Err(reason) => {
                debug_assert!(
                    false,
                    "Removing {:?} from an account ({}) that does not exist: {:?}",
                    stake, identifier, reason,
                );
                self.control.clone()
            }
        };

        Self {
            control,
            assigned: self.assigned.wrapping_sub(stake),
            unassigned: self.unassigned,
        }
    }
}

impl fmt::Debug for StakeControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unassigned: {}, assigned: {}, control: {:?}",
            self.unassigned,
            self.assigned,
            self.control
                .iter()
                .map(|(id, account)| (id.clone(), *account))
                .collect::<Vec<(Identifier, Stake)>>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::StakeControl;
    use crate::{
        account::{self, Identifier},
        rewards::Ratio,
        stake::Stake,
        testing::{utxo::ArbitaryLedgerUtxo, TestGen},
    };
    use quickcheck_macros::quickcheck;
    use std::num::NonZeroU64;

    fn create_stake_control_from(
        assigned: &[(Identifier, Stake)],
        unassigned: Stake,
    ) -> StakeControl {
        let stake_control: StakeControl = assigned
            .iter()
            .fold(StakeControl::new(), |sc, (identifier, stake)| {
                sc.add_to(identifier.clone(), *stake)
            });
        stake_control.add_unassigned(unassigned)
    }

    #[test]
    pub fn empty_stake_control() {
        let random_identifier = TestGen::identifier();
        let stake_control = create_stake_control_from(&[], Stake::zero());

        assert_eq!(stake_control.total(), Stake::zero());
        assert_eq!(stake_control.unassigned(), Stake::zero());
        assert_eq!(stake_control.assigned(), Stake::zero());
        assert_eq!(stake_control.by(&random_identifier), None);
        let expected_ratio = Ratio {
            numerator: 0,
            denominator: NonZeroU64::new(1).unwrap(),
        };
        assert_eq!(stake_control.ratio_by(&random_identifier), expected_ratio);
    }

    #[test]
    pub fn stake_control_only_assigned() {
        let identifier = TestGen::identifier();
        let initial_stake = Stake(100);
        let stake_control =
            create_stake_control_from(&[(identifier.clone(), initial_stake)], Stake::zero());

        assert_eq!(stake_control.total(), initial_stake);
        assert_eq!(stake_control.unassigned(), Stake::zero());
        assert_eq!(stake_control.assigned(), initial_stake);
        assert_eq!(stake_control.by(&identifier).unwrap(), initial_stake);
        let expected_ratio = Ratio {
            numerator: 100,
            denominator: NonZeroU64::new(100).unwrap(),
        };
        assert_eq!(stake_control.ratio_by(&identifier), expected_ratio);
    }

    #[test]
    pub fn stake_control_only_unassigned() {
        let identifier = TestGen::identifier();
        let initial_stake = Stake(100);
        let stake_control = create_stake_control_from(&[], initial_stake);

        assert_eq!(stake_control.total(), initial_stake);
        assert_eq!(stake_control.unassigned(), initial_stake);
        assert_eq!(stake_control.assigned(), Stake::zero());
        assert_eq!(stake_control.by(&identifier), None);
        let expected_ratio = Ratio {
            numerator: 0,
            denominator: NonZeroU64::new(1).unwrap(),
        };
        assert_eq!(stake_control.ratio_by(&identifier), expected_ratio);
    }

    #[test]
    pub fn stake_control_unassigned_and_assigned() {
        let identifier = TestGen::identifier();
        let stake_to_add = Stake(100);

        let stake_control =
            create_stake_control_from(&[(identifier.clone(), stake_to_add)], stake_to_add);

        assert_eq!(stake_control.total(), Stake(200));
        assert_eq!(stake_control.unassigned(), stake_to_add);
        assert_eq!(stake_control.assigned(), stake_to_add);
        assert_eq!(stake_control.by(&identifier), Some(stake_to_add));
        let expected_ratio = Ratio {
            numerator: 100,
            denominator: NonZeroU64::new(100).unwrap(),
        };
        assert_eq!(stake_control.ratio_by(&identifier), expected_ratio);
    }

    #[test]
    pub fn stake_control_remove_part_of_assigned() {
        let identifier = TestGen::identifier();
        let stake_to_add = Stake(100);
        let stake_to_sub = Stake(50);
        let mut stake_control =
            create_stake_control_from(&[(identifier.clone(), stake_to_add)], stake_to_add);
        stake_control = stake_control.remove_from(identifier.clone(), stake_to_sub);

        assert_eq!(stake_control.total(), Stake(150));
        assert_eq!(stake_control.unassigned(), Stake(100));
        assert_eq!(stake_control.assigned(), Stake(50));
        assert_eq!(stake_control.by(&identifier), Some(Stake(50)));
        let expected_ratio = Ratio {
            numerator: 50,
            denominator: NonZeroU64::new(50).unwrap(),
        };
        assert_eq!(stake_control.ratio_by(&identifier), expected_ratio);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "KeyNotFound")]
    pub fn stake_control_remove_non_existing_assigned_debug() {
        let non_existing_identifier = TestGen::identifier();
        let existing_identifier = TestGen::identifier();
        let stake_to_add = Stake(100);

        let stake_control =
            create_stake_control_from(&[(existing_identifier, stake_to_add)], stake_to_add);

        assert_eq!(stake_control.total(), Stake(200));
        let _ = stake_control.remove_from(non_existing_identifier, stake_to_add);
    }

    #[test]
    #[cfg(not(debug_assertions))]
    pub fn stake_control_remove_non_existing_assigned_release() {
        let non_existing_identifier = TestGen::identifier();
        let existing_identifier = TestGen::identifier();
        let stake_to_add = Stake(100);

        let stake_control =
            create_stake_control_from(&[(existing_identifier, stake_to_add.clone())], stake_to_add);

        assert_eq!(stake_control.total(), Stake(200));
        let _ = stake_control.remove_from(non_existing_identifier, stake_to_add);
    }

    #[test]
    pub fn stake_control_remove_all_assigned() {
        let identifier = TestGen::identifier();
        let stake_to_add = Stake(100);

        let mut stake_control =
            create_stake_control_from(&[(identifier.clone(), stake_to_add)], stake_to_add);

        assert_eq!(stake_control.total(), Stake(200));

        stake_control = stake_control.remove_from(identifier.clone(), stake_to_add);

        assert_eq!(stake_control.total(), Stake(100));
        assert_eq!(stake_control.unassigned(), Stake(100));
        assert_eq!(stake_control.assigned(), Stake::zero());
        assert_eq!(stake_control.by(&identifier), Some(Stake::zero()));
        unsafe {
            let expected_ratio = Ratio {
                numerator: 0,
                denominator: NonZeroU64::new_unchecked(0),
            };
            assert_eq!(stake_control.ratio_by(&identifier), expected_ratio);
        }
    }

    #[test]
    pub fn stake_control_remove_unassigned() {
        let identifier = TestGen::identifier();
        let stake_to_add = Stake(100);

        let stake_control =
            create_stake_control_from(&[(identifier.clone(), stake_to_add)], stake_to_add);

        assert_eq!(stake_control.total(), Stake(200));
        assert_eq!(stake_control.unassigned(), stake_to_add);
        assert_eq!(stake_control.assigned(), stake_to_add);
        assert_eq!(stake_control.by(&identifier), Some(stake_to_add));
        let expected_ratio = Ratio {
            numerator: 100,
            denominator: NonZeroU64::new(100).unwrap(),
        };
        assert_eq!(stake_control.ratio_by(&identifier), expected_ratio);
    }

    #[test]
    pub fn stake_control_remove_all() {
        let identifier = TestGen::identifier();
        let stake_to_add = Stake(100);

        let mut stake_control =
            create_stake_control_from(&[(identifier.clone(), stake_to_add)], stake_to_add);

        stake_control = stake_control.remove_from(identifier.clone(), stake_to_add);
        stake_control = stake_control.remove_unassigned(stake_to_add);

        assert_eq!(stake_control.total(), Stake::zero());
        assert_eq!(stake_control.unassigned(), Stake::zero());
        assert_eq!(stake_control.assigned(), Stake::zero());
        assert_eq!(stake_control.by(&identifier), Some(Stake::zero()));
    }

    #[test]
    pub fn stake_control_account_ratio() {
        let first_identifier = TestGen::identifier();
        let second_identifier = TestGen::identifier();
        let stake_to_add = Stake(100);

        let stake_control = create_stake_control_from(
            &[
                (first_identifier.clone(), stake_to_add),
                (second_identifier.clone(), stake_to_add),
            ],
            stake_to_add,
        );

        assert_eq!(stake_control.by(&first_identifier), Some(stake_to_add));
        assert_eq!(stake_control.by(&second_identifier), Some(stake_to_add));

        assert_eq!(stake_control.by(&first_identifier), Some(stake_to_add));
        assert_eq!(stake_control.by(&second_identifier), Some(stake_to_add));

        let expected_ratio = Ratio {
            numerator: 100,
            denominator: NonZeroU64::new(200).unwrap(),
        };
        assert_eq!(stake_control.ratio_by(&first_identifier), expected_ratio);

        let expected_ratio = Ratio {
            numerator: 100,
            denominator: NonZeroU64::new(200).unwrap(),
        };
        assert_eq!(stake_control.ratio_by(&second_identifier), expected_ratio);
    }

    #[quickcheck]
    pub fn stake_control_from_ledger(accounts: account::Ledger, utxos: ArbitaryLedgerUtxo) {
        let stake_control = StakeControl::new_with(&accounts, &utxos.0);
        //verify sum
        let accounts = accounts.get_total_value().unwrap();
        let utxo_or_group = utxos.0.values().map(|x| x.value).sum();
        let expected_sum = accounts
            .checked_add(utxo_or_group)
            .expect("cannot calculate expected total");
        assert_eq!(stake_control.total(), expected_sum.into());
    }
}
