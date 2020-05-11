use chain_impl_mockchain::transaction::Input;

pub struct Conversion {
    pub(crate) ignored: Vec<Input>,
    pub(crate) transactions: Vec<Vec<u8>>,
}

impl Conversion {
    pub fn ignored(&self) -> &[Input] {
        &self.ignored
    }

    pub fn transactions(&self) -> &[Vec<u8>] {
        &self.transactions
    }
}
