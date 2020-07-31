mod blockchain;
mod keygen;
mod password;
mod recovering;
pub mod scheme;
mod states;
mod store;
pub mod transaction;

pub use self::{
    blockchain::Settings,
    password::{Password, ScrubbedBytes},
    recovering::{RecoveryBuilder, RecoveryError},
    transaction::{AccountWitnessBuilder, TransactionBuilder, WitnessBuilder},
};
use crate::scheme::{on_tx_input_and_witnesses, on_tx_output};
use chain_addr::Discrimination;
use chain_impl_mockchain::{
    fragment::{Fragment, FragmentId},
    transaction::{Input, InputEnum},
    value::Value,
};
use hdkeygen::account::Account;
pub use hdkeygen::account::AccountId;
use states::States;
use std::cell::Cell;

pub struct Wallet {
    account: Account,
    state: States<FragmentId, State>,
}

// TODO: remove Cell, it's only because of the signature
// of set_state, I think that function should only be used
// once per "session". But for the time being to avoid a breaking
// change keep it as this.
pub struct State {
    value: Cell<Value>,
    counter: Cell<u32>,
}

pub struct WalletBuildTx<'a> {
    wallet: &'a mut Wallet,
    value: Value,
    counter: u32,
}

impl Wallet {
    pub fn new(account: Account) -> Wallet {
        Wallet {
            account,
            state: States::new(
                FragmentId::zero_hash(),
                State {
                    value: Cell::new(Value::zero()),
                    counter: Cell::new(0),
                },
            ),
        }
    }

    pub fn account_id(&self) -> AccountId {
        self.account.account_id()
    }

    /// set the state counter so we can sync with the blockchain and the
    /// local state
    ///
    /// TODO: some handling to provide information if needed:
    ///
    /// - [ ] check the counter is not regressing?
    /// - [ ] check that there is continuity?
    ///
    /// TODO: change to a constructor/initializator?, or just make it so it resets the state
    ///
    pub fn update_state(&mut self, value: Value, counter: u32) {
        self.state.last_state().1.value.replace(value);
        self.state.last_state().1.counter.replace(counter);
    }

    pub fn value(&self) -> Value {
        self.state.last_state().1.value.get()
    }

    /// confirm a pending transaction
    ///
    /// to only do once it is confirmed a transaction is on chain
    /// and is far enough in the blockchain history to be confirmed
    /// as immutable
    ///
    pub fn confirm(&mut self, fragment_id: &FragmentId) {
        self.state.confirm(fragment_id)
    }

    /// get the confirmed value of the wallet
    pub fn confirmed_value(&self) -> Value {
        self.state.confirmed_state().1.value.get()
    }

    /// get the unconfirmed value of the wallet
    ///
    /// if `None`, it means there is no unconfirmed state of the wallet
    /// and the value can be known from `confirmed_value`.
    ///
    /// The returned value is the value we expect to see at some point on
    /// chain once all transactions are on chain confirmed.
    pub fn unconfirmed_value(&self) -> Option<Value> {
        let (k, s, _) = self.state.last_state();
        let (kk, _) = self.state.confirmed_state();

        if k == kk {
            None
        } else {
            Some(s.value.get())
        }
    }

    pub fn new_transaction(&mut self, value: Value) -> WalletBuildTx {
        let (_, state, _) = self.state.last_state();
        let counter = state.counter.get();
        WalletBuildTx {
            wallet: self,
            value,
            counter,
        }
    }

    pub fn check_fragment(&mut self, fragment_id: &FragmentId, fragment: &Fragment) -> bool {
        if self.state.contains(fragment_id) {
            return true;
        }

        let (_, state, _) = self.state.last_state();

        let mut new_value = state.value.get();

        let mut increment_counter = false;

        match fragment {
            Fragment::Initial(_config_params) => {}
            Fragment::UpdateProposal(_update_proposal) => {}
            Fragment::UpdateVote(_signed_update) => {}
            Fragment::OldUtxoDeclaration(_utxos) => {}
            _ => {
                on_tx_input_and_witnesses(fragment, |(input, witness)| {
                    if let InputEnum::AccountInput(id, value) = input.to_enum() {
                        if self.account.account_id().as_ref() == id.as_ref() {
                            new_value = value.checked_sub(new_value).expect("value overflow");
                        }
                    }

                    increment_counter = true;

                    // TODO: check monotonicity by signing and comparing
                    // if let Witness::Account(witness) = witness {
                    //
                    // }
                });
                on_tx_output(fragment, |(_, output)| {
                    if output.address == self.account_id().address(Discrimination::Production) {
                        new_value = new_value.checked_add(output.value).unwrap();
                    }
                })
            }
        };

        let counter = if increment_counter {
            state
                .counter
                .replace(state.counter.get().checked_add(1).unwrap())
        } else {
            state.counter.get()
        };

        let new_state = State {
            counter: Cell::new(counter),
            value: Cell::new(new_value),
        };

        self.state.push(*fragment_id, new_state);

        false
    }
}

impl<'a> WalletBuildTx<'a> {
    pub fn input(&self) -> Input {
        Input::from_account_public_key(self.wallet.account_id().into(), self.value)
    }

    pub fn witness_builder(&self) -> AccountWitnessBuilder {
        let mut account = self.wallet.account.clone();
        account.set_counter(self.counter);

        transaction::AccountWitnessBuilder(account)
    }

    pub fn add_fragment_id(self, fragment_id: FragmentId) {
        self.wallet.state.push(
            fragment_id,
            State {
                value: Cell::new(self.value),
                counter: Cell::new(self.counter.checked_add(1).unwrap()),
            },
        );
    }
}
