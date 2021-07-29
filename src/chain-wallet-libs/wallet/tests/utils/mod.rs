use chain_impl_mockchain::{
    block::Block,
    fragment::{Fragment, FragmentRaw},
    ledger::{Error as LedgerError, Ledger},
};
use chain_ser::mempack::{ReadBuf, Readable as _};
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
        let mut block0_bytes = ReadBuf::from(block0_bytes.as_ref());
        let block0 = Block::read(&mut block0_bytes).expect("valid block0");
        let hh = block0.header.id();
        let ledger = Ledger::new(hh, block0.fragments()).unwrap();

        Self { block0, ledger }
    }

    #[allow(dead_code)]
    pub fn initial_contents(&self) -> impl Iterator<Item = &'_ Fragment> {
        self.block0.contents.iter()
    }

    pub fn settings(&self) -> Result<Settings, LedgerError> {
        Settings::new(&self.block0)
    }

    pub fn apply_fragments<'a, F>(&'a mut self, fragments: F) -> Result<(), LedgerError>
    where
        F: IntoIterator<Item = &'a FragmentRaw>,
    {
        let ledger_params = self.ledger.get_ledger_parameters();
        let block_date = self.ledger.date();
        let mut new_ledger = self.ledger.clone();
        for fragment in fragments {
            let fragment = Fragment::from_raw(fragment).unwrap();
            new_ledger = self
                .ledger
                .apply_fragment(&ledger_params, &fragment, block_date)?;
        }

        self.ledger = new_ledger;

        Ok(())
    }
}
