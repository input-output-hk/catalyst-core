use super::transaction::AccountWitnessBuilder;
use crate::scheme::{on_tx_input_and_witnesses, on_tx_output};
use crate::states::{States, Status};
use chain_crypto::{Ed25519, Ed25519Extended, PublicKey, SecretKey};
use chain_impl_mockchain::{
    fragment::{Fragment, FragmentId},
    transaction::{Input, InputEnum},
    value::Value,
};
pub use hdkeygen::account::AccountId;
use hdkeygen::account::{Account, SEED};

pub struct Wallet {
    account: EitherAccount,
    state: States<FragmentId, State>,
}

pub struct State {
    value: Value,
    counter: u32,
}

pub struct WalletBuildTx<'a> {
    wallet: &'a mut Wallet,
    value: Value,
    counter: u32,
}

enum EitherAccount {
    Seed(Account<SEED>),
    Extended(Account<SecretKey<Ed25519Extended>>),
}

impl Wallet {
    pub fn new_from_seed(seed: SEED) -> Wallet {
        Wallet {
            account: EitherAccount::Seed(Account::from_seed(seed)),
            state: States::new(
                FragmentId::zero_hash(),
                State {
                    value: Value::zero(),
                    counter: 0,
                },
            ),
        }
    }

    pub fn new_from_key(key: SecretKey<Ed25519Extended>) -> Wallet {
        Wallet {
            account: EitherAccount::Extended(Account::from_secret_key(key)),
            state: States::new(
                FragmentId::zero_hash(),
                State {
                    value: Value::zero(),
                    counter: 0,
                },
            ),
        }
    }

    pub fn account_id(&self) -> AccountId {
        match &self.account {
            EitherAccount::Extended(account) => account.account_id(),
            EitherAccount::Seed(account) => account.account_id(),
        }
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
        self.state = States::new(FragmentId::zero_hash(), State { value, counter });
    }

    pub fn value(&self) -> Value {
        self.state.last_state().1.value
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

    /// get all the pending transactions of the wallet
    ///
    /// If empty it means there's no pending transactions waiting confirmation
    ///
    pub fn pending_transactions(&self) -> impl Iterator<Item = &FragmentId> {
        self.state.iter().filter_map(|(k, _, status)| {
            if status == Status::Pending {
                Some(k)
            } else {
                None
            }
        })
    }

    /// get the confirmed value of the wallet
    pub fn confirmed_value(&self) -> Value {
        self.state.confirmed_state().1.value
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
            Some(s.value)
        }
    }

    pub fn new_transaction(&mut self, value: Value) -> WalletBuildTx {
        let (_, state, _) = self.state.last_state();
        let counter = state.counter;
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

        let mut new_value = state.value;

        let mut increment_counter = false;
        let mut at_least_one_output = false;

        match fragment {
            Fragment::Initial(_config_params) => {}
            Fragment::UpdateProposal(_update_proposal) => {}
            Fragment::UpdateVote(_signed_update) => {}
            Fragment::OldUtxoDeclaration(_utxos) => {}
            _ => {
                on_tx_input_and_witnesses(fragment, |(input, _witness)| {
                    if let InputEnum::AccountInput(id, value) = input.to_enum() {
                        if self.account_id().as_ref() == id.as_ref() {
                            new_value = value.checked_sub(new_value).expect("value overflow");
                        }
                        increment_counter = true;
                    }

                    // TODO: check monotonicity by signing and comparing
                    // if let Witness::Account(witness) = witness {
                    //
                    // }
                });
                on_tx_output(fragment, |(_, output)| {
                    if output
                        .address
                        .public_key()
                        .map(|pk| *pk == Into::<PublicKey<Ed25519>>::into(self.account_id()))
                        .unwrap_or(false)
                    {
                        new_value = new_value.checked_add(output.value).unwrap();
                        at_least_one_output = true;
                    }
                })
            }
        };

        let counter = if increment_counter {
            state
                .counter
                .checked_add(1)
                .expect("account counter overflow")
        } else {
            state.counter
        };

        let new_state = State {
            counter,
            value: new_value,
        };

        self.state.push(*fragment_id, new_state);

        at_least_one_output || increment_counter
    }
}

impl<'a> WalletBuildTx<'a> {
    pub fn input(&self) -> Input {
        Input::from_account_public_key(self.wallet.account_id().into(), self.value)
    }

    pub fn witness_builder(&self) -> AccountWitnessBuilder {
        match &self.wallet.account {
            EitherAccount::Seed(account) => crate::transaction::AccountWitnessBuilder::Ed25519(
                account.secret_key(),
                self.counter.into(),
            ),
            EitherAccount::Extended(account) => {
                crate::transaction::AccountWitnessBuilder::Ed25519Extended(
                    account.secret_key(),
                    self.counter.into(),
                )
            }
        }
    }

    pub fn add_fragment_id(self, fragment_id: FragmentId) {
        self.wallet.state.push(
            fragment_id,
            State {
                value: self.value,
                counter: self.counter.checked_add(1).unwrap(),
            },
        );
    }
}
