use chain_impl_mockchain::{fragment::FragmentRaw, transaction::Input};

pub struct Conversion {
    pub(crate) ignored: Vec<Input>,
    pub(crate) transactions: Vec<FragmentRaw>,
}

impl Conversion {
    pub fn ignored(&self) -> &[Input] {
        &self.ignored
    }

    pub fn transactions(&self) -> &[FragmentRaw] {
        &self.transactions
    }
}
