use chain_impl_mockchain::transaction::{Input, NoExtra, Transaction};

pub struct Conversion {
    pub(crate) ignored: Vec<Input>,
    pub(crate) transactions: Vec<Transaction<NoExtra>>,
}

impl Conversion {
    pub fn ignored(&self) -> &[Input] {
        &self.ignored
    }

    pub fn transactions(&self) -> &[Transaction<NoExtra>] {
        &self.transactions
    }
}
