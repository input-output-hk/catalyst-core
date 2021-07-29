use crate::{Conversion, Error, Proposal};
use chain_core::property::Serialize as _;
use chain_crypto::SecretKey;
use chain_impl_mockchain::{
    block::{Block, BlockDate},
    fragment::{Fragment, FragmentId},
    transaction::Input,
    value::Value,
    vote::Choice,
};
use chain_ser::mempack::{ReadBuf, Readable as _};
use wallet::{AccountId, Settings};

/// the wallet
///
/// * use the `recover` function to recover the wallet from the mnemonics/password;
/// * use the `retrieve_funds` to retrieve initial funds (if necessary) from the block0;
///   then you can use `total_value` to see how much was recovered from the initial block0;
///
pub struct Wallet {
    account: wallet::Wallet,
    free_keys: wallet::scheme::freeutxo::Wallet,
}

impl Wallet {
    pub fn account(&self, discrimination: chain_addr::Discrimination) -> chain_addr::Address {
        self.account.account_id().address(discrimination)
    }

    pub fn id(&self) -> AccountId {
        self.account.account_id()
    }

    /// retrieve a wallet from the given mnemonics, password and protocol magic
    ///
    /// You can also use this function to recover a wallet even after you have transferred all the
    /// funds to the new format (see the _convert_ function)
    ///
    /// The recovered wallet will be returned in `wallet_out`.
    ///
    /// # parameters
    ///
    /// * mnemonics: a null terminated utf8 string (already normalized NFKD) in english; *
    /// password: pointer to the password (in bytes, can be UTF8 string or a bytes of anything);
    /// this value is optional and passing a null pointer will result in no password;
    ///
    /// # errors
    ///
    /// The function may fail if:
    ///
    /// * the mnemonics are not valid (invalid length or checksum);
    ///
    pub fn recover(mnemonics: &str, password: &[u8]) -> Result<Self, Error> {
        let builder = wallet::RecoveryBuilder::new();

        let builder = builder
            .mnemonics(&bip39::dictionary::ENGLISH, mnemonics)
            .map_err(|err| Error::invalid_input("mnemonics").with(err))?;

        let builder = if !password.is_empty() {
            todo!()
        } else {
            builder
        };

        let account = builder
            .build_wallet()
            .expect("build the account cannot fail as expected");

        let free_keys = builder
            .build_free_utxos()
            .expect("build without free keys cannot fail");

        Ok(Wallet { account, free_keys })
    }

    /// Retrieve a wallet from a list of free keys used as utxo's
    ///
    /// You can also use this function to recover a wallet even after you have
    /// transferred all the funds to the new format (see the _convert_ function)
    ///
    /// Parameters
    ///
    /// * account_key: the private key used for voting
    /// * keys: single keys used as utxo inputs
    ///
    /// # Errors
    ///
    /// The function may fail if:
    ///
    /// TODO
    ///
    pub fn recover_free_keys(account_key: &[u8], keys: &[[u8; 64]]) -> Result<Self, Error> {
        let builder = wallet::RecoveryBuilder::new();

        let builder = builder.account_secret_key(SecretKey::from_binary(account_key).unwrap());

        let builder = keys.iter().fold(builder, |builder, key| {
            builder.add_key(SecretKey::from_binary(key.as_ref()).unwrap())
        });

        let free_keys = builder
            .build_free_utxos()
            .map_err(|err| Error::invalid_input("todo").with(err))?;

        let account = builder
            .build_wallet()
            .expect("build the account cannot fail as expected");

        Ok(Wallet { account, free_keys })
    }

    /// retrieve funds from bip44 hd sequential wallet in the given block0 (or any other blocks).
    ///
    /// Execute this function then you can check who much funds you have retrieved from the given
    /// block.
    ///
    /// this function may take sometimes so it is better to only call this function if needed.
    ///
    /// # Errors
    ///
    /// * the block is not valid (cannot be decoded)
    ///
    pub fn retrieve_funds(&mut self, block0_bytes: &[u8]) -> Result<wallet::Settings, Error> {
        let mut block0_bytes = ReadBuf::from(block0_bytes);
        let block0 =
            Block::read(&mut block0_bytes).map_err(|e| Error::invalid_input("block0").with(e))?;

        let settings = wallet::Settings::new(&block0).unwrap();
        for fragment in block0.contents.iter() {
            self.free_keys.check_fragment(&fragment.hash(), fragment);
            self.account.check_fragment(&fragment.hash(), fragment);

            self.confirm_transaction(fragment.hash());
        }

        Ok(settings)
    }

    /// once funds have been retrieved with `iohk_jormungandr_wallet_retrieve_funds`
    /// it is possible to convert all existing funds to the new wallet.
    ///
    /// The returned arrays are transactions to send to the network in order to do the
    /// funds conversion.
    ///
    /// Don't forget to call `iohk_jormungandr_wallet_delete_conversion` to
    /// properly free the memory
    ///
    /// # Safety
    ///
    /// This function dereference raw pointers (wallet, settings and conversion_out). Even though
    /// the function checks if the pointers are null. Mind not to put random values
    /// in or you may see unexpected behaviors
    ///
    pub fn convert(&mut self, settings: Settings, valid_until: &BlockDate) -> Conversion {
        let address = self.account.account_id().address(settings.discrimination());

        let mut raws = Vec::new();
        let mut ignored = Vec::new();

        let account = &mut self.account;
        let mut for_each = |fragment: Fragment, mut ignored_inputs: Vec<Input>| {
            raws.push(fragment.serialize_as_vec().unwrap());
            ignored.append(&mut ignored_inputs);
            account.check_fragment(&fragment.hash(), &fragment);
        };

        for (fragment, ignored) in wallet::transaction::dump_free_utxo(
            &settings,
            &address,
            &mut self.free_keys,
            *valid_until,
        ) {
            for_each(fragment, ignored)
        }

        Conversion {
            ignored,
            transactions: raws,
        }
    }

    /// use this function to confirm a transaction has been properly received
    ///
    /// This function will automatically update the state of the wallet
    pub fn confirm_transaction(&mut self, id: FragmentId) {
        self.free_keys.confirm(&id);
        self.account.confirm(&id);
    }

    /// get access to all the pending transaction
    ///
    /// TODO: this might need to be updated to have a more user friendly
    ///       API. Currently do this for simplicity
    pub fn pending_transactions(&self) -> Vec<FragmentId> {
        let mut check = std::collections::HashSet::new();
        let mut result = vec![];

        for id in self.free_keys.pending_transactions() {
            if check.insert(*id) {
                result.push(*id);
            }
        }

        for id in self.account.pending_transactions() {
            if check.insert(*id) {
                result.push(*id);
            }
        }

        result
    }

    /// remove a given pending transaction returning the associated Inputs
    /// that were used for this transaction
    pub fn remove_pending_transaction(&mut self, _id: &FragmentId) -> Option<Vec<Input>> {
        // there are no cordova bindings for this, so this todo is not that important right now
        // and I'm not that sure what's the best api here.
        todo!("pending transactions");
    }

    /// get the current spending counter
    ///
    pub fn spending_counter(&self) -> u32 {
        self.account.spending_counter().into()
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
        self.free_keys
            .utxos()
            .total_value()
            .saturating_add(self.account.value())
    }

    /// update the wallet account state
    ///
    /// this is the value retrieved from any jormungandr endpoint that allows to query
    /// for the account state. It gives the value associated to the account as well as
    /// the counter.
    ///
    /// It is important to be sure to have an updated wallet state before doing any
    /// transactions otherwise future transactions may fail to be accepted by any
    /// nodes of the blockchain because of invalid signature state.
    ///
    pub fn set_state(&mut self, value: Value, counter: u32) {
        self.account.update_state(value, counter.into())
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
    ) -> Result<Box<[u8]>, Error> {
        let payload = if let Some(payload) = proposal.vote(choice) {
            payload
        } else {
            return Err(Error::wallet_vote_range());
        };

        let mut builder = wallet::TransactionBuilder::new(&settings, payload, *valid_until);

        let value = builder.estimate_fee_with(1, 0);

        let account_tx_builder = self
            .account
            .new_transaction(value)
            .map_err(|_| Error::not_enough_funds())?;

        let input = account_tx_builder.input();
        let witness_builder = account_tx_builder.witness_builder();

        builder.add_input(input, witness_builder);

        let tx = builder
            .finalize_tx(())
            .map_err(|e| Error::wallet_transaction().with(e))?;

        let fragment = Fragment::VoteCast(tx);
        let raw = fragment.to_raw();
        let id = raw.id();

        account_tx_builder.add_fragment_id(id);

        Ok(raw.serialize_as_vec().unwrap().into_boxed_slice())
    }
}
