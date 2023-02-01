use crate::{Error, Proposal};
use chain_core::property::Serialize as _;
use chain_crypto::SecretKey;
use chain_impl_mockchain::{
    account::SpendingCounterIncreasing,
    block::BlockDate,
    fragment::{Fragment, FragmentId},
    value::Value,
    vote::Choice,
};
use wallet::{transaction::WitnessInput, AccountId, Settings};

/// the wallet
///
/// * use the `recover` function to recover the wallet from the mnemonics/password;
/// * use the `retrieve_funds` to retrieve initial funds (if necessary) from the block0;
///   then you can use `total_value` to see how much was recovered from the initial block0;
///
pub struct Wallet {
    account: wallet::Wallet,
}

impl Wallet {
    /// Returns address of the account with the given chain discrimination.
    pub fn account(&self, discrimination: chain_addr::Discrimination) -> chain_addr::Address {
        self.account.account_id().address(discrimination)
    }

    pub fn id(&self) -> AccountId {
        self.account.account_id()
    }

    /// Retrieve a wallet from a key used as utxo's
    ///
    /// You can also use this function to recover a wallet even after you have
    /// transferred all the funds to the new format
    ///
    /// Parameters
    ///
    /// * `account_key`: the private key used for voting
    ///
    /// # Errors
    ///
    /// The function may fail if:
    ///
    /// TODO
    ///
    pub fn recover_free_keys(account_key: &[u8]) -> Result<Self, Error> {
        let account = wallet::Wallet::new_from_key(SecretKey::from_binary(account_key).unwrap());

        Ok(Wallet { account })
    }

    /// use this function to confirm a transaction has been properly received
    ///
    /// This function will automatically update the state of the wallet
    pub fn confirm_transaction(&mut self, id: FragmentId) {
        self.account.confirm(&id);
    }

    /// get the current spending counter
    ///
    pub fn spending_counter(&self) -> [u32; SpendingCounterIncreasing::LANES] {
        let spending_counters = self.account.spending_counter();
        [
            spending_counters[0].into(),
            spending_counters[1].into(),
            spending_counters[2].into(),
            spending_counters[3].into(),
            spending_counters[4].into(),
            spending_counters[5].into(),
            spending_counters[6].into(),
            spending_counters[7].into(),
        ]
    }

    /// get the total value in the wallet
    ///
    /// make sure to call `retrieve_funds` prior to calling this function
    /// otherwise you will always have `0`
    ///
    /// Once a conversion has been performed, this value can be use to display
    /// how much the wallet started with or retrieved from the chain.
    ///
    pub fn total_value(&self) -> Value {
        self.account.value()
    }

    /// Update the wallet's account state.
    ///
    /// The values to update the account state with can be retrieved from a
    /// Jormungandr API endpoint. It sets the balance value on the account
    /// as well as the current spending counter.
    ///
    /// It is important to be sure to have an up to date wallet state
    /// before doing any transactions, otherwise future transactions may fail
    /// to be accepted by the blockchain nodes because of an invalid witness
    /// signature.
    pub fn set_state(
        &mut self,
        value: Value,
        counters: [u32; SpendingCounterIncreasing::LANES],
    ) -> Result<(), Error> {
        self.account
            .set_state(
                value,
                [
                    counters[0].into(),
                    counters[1].into(),
                    counters[2].into(),
                    counters[3].into(),
                    counters[4].into(),
                    counters[5].into(),
                    counters[6].into(),
                    counters[7].into(),
                ],
            )
            .map_err(|_| Error::invalid_spending_counters())
    }

    /// Cast a vote
    ///
    /// This function outputs a fragment containing a voting transaction.
    ///
    /// # Parameters
    ///
    /// * `settings` - ledger settings.
    /// * `proposal` - proposal information including the range of values
    ///   allowed in `choice`.
    /// * `choice` - the option to vote for.
    ///
    /// # Errors
    ///
    /// The error is returned when `choice` does not fall withing the range of
    /// available choices specified in `proposal`.
    pub fn vote(
        &mut self,
        settings: Settings,
        proposal: &Proposal,
        choice: Choice,
        valid_until: &BlockDate,
        lane: u8,
    ) -> Result<Box<[u8]>, Error> {
        let payload = if let Some(payload) = proposal.vote(choice) {
            payload
        } else {
            return Err(Error::wallet_vote_range());
        };

        let mut builder = wallet::TransactionBuilder::new(settings, payload, *valid_until);

        let value = builder.estimate_fee_with(1, 0);

        let secret_key = self.account.secret_key();
        let account_tx_builder = self
            .account
            .new_transaction(value, lane)
            .map_err(|_| Error::not_enough_funds())?;

        let input = account_tx_builder.input();
        let witness_builder = account_tx_builder.witness_builder();

        builder.add_input(input, witness_builder);

        let tx = builder
            .finalize_tx((), vec![WitnessInput::SecretKey(secret_key)])
            .map_err(|e| Error::wallet_transaction().with(e))?;

        let fragment = Fragment::VoteCast(tx);
        let id = fragment.hash();

        account_tx_builder.add_fragment_id(id);

        Ok(fragment.serialize_as_vec().unwrap().into_boxed_slice())
    }
}
