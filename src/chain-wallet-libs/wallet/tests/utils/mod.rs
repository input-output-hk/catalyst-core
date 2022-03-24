use chain_impl_mockchain::{
    block::Block,
    fragment::Fragment,
    ledger::{Error as LedgerError, Ledger},
    value::Value,
};
use chain_ser::{deser::DeserializeFromSlice, packer::Codec};
use wallet::Settings;

pub struct State {
    block0: Block,
    pub ledger: Ledger,
}

impl State {
    pub fn new<B>(block0_bytes: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        let block0 = Block::deserialize_from_slice(&mut Codec::new(block0_bytes.as_ref()))
            .expect("valid block0");
        let hh = block0.header().id();
        let ledger = Ledger::new(hh, block0.fragments()).unwrap();

        Self { block0, ledger }
    }

    #[allow(dead_code)]
    pub fn initial_contents(&self) -> impl Iterator<Item = &'_ Fragment> {
        self.block0.contents().iter()
    }

    pub fn settings(&self) -> Result<Settings, LedgerError> {
        Settings::new(&self.block0)
    }

    pub fn apply_fragments<'a, F>(&'a mut self, fragments: F) -> Result<(), LedgerError>
    where
        F: IntoIterator<Item = &'a Fragment>,
    {
        let ledger_params = self.ledger.get_ledger_parameters();
        let block_date = self.ledger.date();
        let mut new_ledger = self.ledger.clone();
        for fragment in fragments {
            new_ledger = self
                .ledger
                .apply_fragment(&ledger_params, fragment, block_date)?;
        }

        self.ledger = new_ledger;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_account_state(
        &self,
        account_id: wallet::AccountId,
    ) -> Option<(chain_impl_mockchain::account::SpendingCounter, Value)> {
        self.ledger
            .accounts()
            .get_state(&chain_crypto::PublicKey::from(account_id).into())
            .ok()
            .map(|account_state| {
                (
                    account_state.spending.get_valid_counter(),
                    account_state.value,
                )
            })
    }
}
