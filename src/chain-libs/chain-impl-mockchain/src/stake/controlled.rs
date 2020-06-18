use crate::{
    account::{self, Identifier},
    rewards::Ratio,
    stake::Stake,
    utxo,
};
use chain_addr::{Address, Kind};
use imhamt::Hamt;
use std::{collections::hash_map::DefaultHasher, num::NonZeroU64};

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
