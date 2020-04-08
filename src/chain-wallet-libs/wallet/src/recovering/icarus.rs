use hdkeygen::bip44::Wallet;

pub struct RecoveringIcarus {
    wallet: Wallet,
    value_total: u64,
}

impl RecoveringIcarus {
    pub(crate) fn new(wallet: Wallet) -> Self {
        Self {
            wallet,
            value_total: 0,
        }
    }
}
