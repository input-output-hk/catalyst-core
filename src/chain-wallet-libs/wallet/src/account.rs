use super::transaction::AccountWitnessBuilder;
use crate::scheme::{on_tx_input_and_witnesses, on_tx_output};
use crate::states::States;
use chain_crypto::{Ed25519, Ed25519Extended, PublicKey, SecretKey};
use chain_impl_mockchain::accounting::account::{spending, SpendingCounterIncreasing};
use chain_impl_mockchain::{
    account::SpendingCounter,
    fragment::{Fragment, FragmentId},
    transaction::{Input, InputEnum},
    value::Value,
};
pub use hdkeygen::account::AccountId;
use hdkeygen::account::{Account, Seed};
use thiserror::Error;

pub struct Wallet {
    account: EitherAccount,
    state: States<FragmentId, State>,
}

#[derive(Debug, Default)]
pub struct State {
    value: Value,
    counters: SpendingCounterIncreasing,
}

pub struct WalletBuildTx<'a> {
    wallet: &'a mut Wallet,
    needed_input: Value,
    next_value: Value,
    current_counter: SpendingCounter,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("not enough funds, needed {needed:?}, available {current:?}")]
    NotEnoughFunds { current: Value, needed: Value },
    #[error("invalid lane for spending counter")]
    InvalidLane,
    #[error("spending counter does not match current state")]
    NonMonotonicSpendingCounter,
    #[error(transparent)]
    SpendingCounters(#[from] spending::Error),
}

pub enum EitherAccount {
    Seed(Account<Ed25519>),
    Extended(Account<Ed25519Extended>),
}

impl EitherAccount {
    pub fn new_from_seed(seed: Seed) -> Self {
        EitherAccount::Seed(Account::from_seed(seed))
    }

    pub fn new_from_key(key: SecretKey<Ed25519Extended>) -> Self {
        EitherAccount::Extended(Account::from_secret_key(key))
    }

    pub fn account_id(&self) -> AccountId {
        match self {
            EitherAccount::Extended(account) => account.account_id(),
            EitherAccount::Seed(account) => account.account_id(),
        }
    }

    pub fn witness_builder(&self, spending_counter: SpendingCounter) -> AccountWitnessBuilder {
        match &self {
            EitherAccount::Seed(account) => crate::transaction::AccountWitnessBuilder::Ed25519(
                account.secret().clone(),
                spending_counter,
            ),
            EitherAccount::Extended(account) => {
                crate::transaction::AccountWitnessBuilder::Ed25519Extended(
                    account.secret().clone(),
                    spending_counter,
                )
            }
        }
    }
}

impl Wallet {
    pub fn new_from_seed(seed: Seed) -> Self {
        Wallet {
            account: EitherAccount::new_from_seed(seed),
            state: States::new(FragmentId::zero_hash(), Default::default()),
        }
    }

    pub fn new_from_key(key: SecretKey<Ed25519Extended>) -> Self {
        Wallet {
            account: EitherAccount::new_from_key(key),
            state: States::new(FragmentId::zero_hash(), Default::default()),
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
    pub fn set_state(
        &mut self,
        value: Value,
        counters: [SpendingCounter; SpendingCounterIncreasing::LANES],
    ) -> Result<(), Error> {
        let counters = SpendingCounterIncreasing::new_from_counters(counters)?;

        self.state = States::new(FragmentId::zero_hash(), State { value, counters });

        Ok(())
    }

    pub fn spending_counter(&self) -> [SpendingCounter; SpendingCounterIncreasing::LANES] {
        self.state
            .last_state()
            .state()
            .counters
            .get_valid_counters()
    }

    pub fn value(&self) -> Value {
        self.state.last_state().state().value
    }

    /// confirm a pending transaction
    ///
    /// to only do once it is confirmed a transaction is on chain
    /// and is far enough in the blockchain history to be confirmed
    /// as immutable
    ///
    pub fn confirm(&mut self, fragment_id: &FragmentId) {
        self.state.confirm(fragment_id);
    }

    /// get all the pending transactions of the wallet
    ///
    /// If empty it means there's no pending transactions waiting confirmation
    ///
    pub fn pending_transactions(&self) -> impl Iterator<Item = &FragmentId> {
        self.state.unconfirmed_states().map(|(k, _)| k)
    }

    /// get the confirmed value of the wallet
    pub fn confirmed_value(&self) -> Value {
        self.state.confirmed_state().state().value
    }

    /// get the unconfirmed value of the wallet
    ///
    /// if `None`, it means there is no unconfirmed state of the wallet
    /// and the value can be known from `confirmed_value`.
    ///
    /// The returned value is the value we expect to see at some point on
    /// chain once all transactions are on chain confirmed.
    pub fn unconfirmed_value(&self) -> Option<Value> {
        let s = self.state.last_state();

        if s.is_confirmed() {
            None
        } else {
            Some(s.state().value)
        }
    }

    pub fn new_transaction(
        &mut self,
        needed_input: Value,
        lane: u8,
    ) -> Result<WalletBuildTx, Error> {
        let state = self.state.last_state().state();

        let current_counter = *state
            .counters
            .get_valid_counters()
            .get(lane as usize)
            .ok_or(Error::InvalidLane)?;

        let next_value =
            state
                .value
                .checked_sub(needed_input)
                .map_err(|_| Error::NotEnoughFunds {
                    current: state.value,
                    needed: needed_input,
                })?;

        Ok(WalletBuildTx {
            wallet: self,
            needed_input,
            current_counter,
            next_value,
        })
    }

    pub fn check_fragment(
        &mut self,
        fragment_id: &FragmentId,
        fragment: &Fragment,
    ) -> Result<bool, Error> {
        if self.state.contains(fragment_id) {
            return Ok(true);
        }

        let state = self.state.last_state().state();

        let mut new_value = state.value;

        let mut increment_counter = None;
        let mut at_least_one_output = false;

        match fragment {
            Fragment::Initial(_config_params) => {}
            Fragment::UpdateProposal(_update_proposal) => {}
            Fragment::UpdateVote(_signed_update) => {}
            Fragment::OldUtxoDeclaration(_utxos) => {}
            _ => {
                on_tx_input_and_witnesses(fragment, |(input, witness)| {
                    if let InputEnum::AccountInput(id, input_value) = input.to_enum() {
                        if self.account_id().as_ref() == id.as_ref() {
                            new_value = new_value.checked_sub(input_value).expect("value overflow");

                            match witness {
                                chain_impl_mockchain::transaction::Witness::Account(
                                    spending,
                                    _,
                                ) => increment_counter = Some(spending),
                                _ => unreachable!("wrong witness type in account input"),
                            }
                        }
                    }
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

        let counters = if let Some(counter) = increment_counter {
            let mut new = state.counters.clone();
            new.next_verify(counter)
                .map_err(|_| Error::NonMonotonicSpendingCounter)?;
            new
        } else {
            state.counters.clone()
        };

        let new_state = State {
            counters,
            value: new_value,
        };

        self.state.push(*fragment_id, new_state);

        Ok(at_least_one_output || increment_counter.is_some())
    }
}

impl<'a> WalletBuildTx<'a> {
    pub fn input(&self) -> Input {
        Input::from_account_public_key(self.wallet.account_id().into(), self.needed_input)
    }

    pub fn witness_builder(&self) -> AccountWitnessBuilder {
        self.wallet.account.witness_builder(self.current_counter)
    }

    pub fn add_fragment_id(self, fragment_id: FragmentId) {
        let mut counters = self.wallet.state.last_state().state().counters.clone();

        // the counter comes from the current state, so this shouldn't panic
        counters.next_verify(self.current_counter).unwrap();

        self.wallet.state.push(
            fragment_id,
            State {
                value: self.next_value,
                counters,
            },
        );
    }
}
