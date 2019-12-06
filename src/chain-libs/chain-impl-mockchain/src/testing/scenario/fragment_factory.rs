use crate::{
    accounting::account::{DelegationRatio, DelegationType},
    certificate::Certificate,
    certificate::PoolId,
    fragment::Fragment,
    key::Hash,
    testing::{
        builders::{
            build_no_stake_delegation, build_owner_stake_delegation,
            build_owner_stake_full_delegation, build_stake_delegation_cert,
            build_stake_pool_registration_cert, build_stake_pool_retirement_cert, TestTxBuilder,
            TestTxCertBuilder,
        },
        data::{StakePool, Wallet},
        ledger::TestLedger,
    },
    value::Value,
};

#[derive(Clone, Debug)]
pub struct FragmentFactory {
    pub block0_hash: Hash,
}

impl FragmentFactory {
    pub fn new(block0_hash: Hash) -> Self {
        FragmentFactory { block0_hash }
    }

    pub fn transaction(
        &self,
        from: &Wallet,
        to: &Wallet,
        mut test_ledger: &mut TestLedger,
        funds: u64,
    ) -> Fragment {
        TestTxBuilder::new(&test_ledger.block0_hash)
            .move_funds(
                &mut test_ledger,
                &from.as_account(),
                &to.as_account(),
                &Value(funds),
            )
            .get_fragment()
    }

    pub fn stake_pool_registration(
        &self,
        funder: &Wallet,
        stake_pool: &StakePool,
        ledger: &mut TestLedger,
    ) -> Fragment {
        let cert = build_stake_pool_registration_cert(&stake_pool.info());
        self.transaction_with_cert(&[funder], cert, ledger)
    }

    pub fn delegation(
        &self,
        from: &Wallet,
        stake_pool: &StakePool,
        ledger: &mut TestLedger,
    ) -> Fragment {
        let cert = build_stake_delegation_cert(&stake_pool.info(), &from.as_account_data());
        self.transaction_with_cert(&[from], cert, ledger)
    }

    pub fn delegation_different_funder(
        &self,
        funder: &Wallet,
        delegation: &Wallet,
        stake_pool: &StakePool,
        ledger: &mut TestLedger,
    ) -> Fragment {
        let cert = build_stake_delegation_cert(&stake_pool.info(), &delegation.as_account_data());
        self.transaction_with_cert(&[funder], cert, ledger)
    }

    pub fn delegation_remove(&self, from: &Wallet, ledger: &mut TestLedger) -> Fragment {
        let cert = build_no_stake_delegation();
        self.transaction_with_cert(&[from], cert, ledger)
    }

    pub fn delegation_to_many(
        &self,
        from: &Wallet,
        distribution: &[(&StakePool, u8)],
        ledger: &mut TestLedger,
    ) -> Fragment {
        let pools_ratio_sum: u8 = distribution.iter().map(|(_st, ratio)| *ratio as u8).sum();
        let pools: Vec<(PoolId, u8)> = distribution
            .iter()
            .map(|(st, ratio)| (st.info().to_id().clone(), *ratio))
            .collect();

        let delegation_ratio = DelegationRatio::new(pools_ratio_sum, pools);
        let delegation_type = DelegationType::Ratio(delegation_ratio.unwrap());
        let cert = build_owner_stake_delegation(delegation_type);
        self.transaction_with_cert(&[from], cert, ledger)
    }

    pub fn owner_delegation(
        &self,
        from: &Wallet,
        stake_pool: &StakePool,
        ledger: &mut TestLedger,
    ) -> Fragment {
        let cert = build_owner_stake_full_delegation(stake_pool.id());
        self.transaction_with_cert(&[from], cert, ledger)
    }

    pub fn stake_pool_retire(
        &self,
        owners: &[&Wallet],
        stake_pool: &StakePool,
        ledger: &mut TestLedger,
    ) -> Fragment {
        let certificate = build_stake_pool_retirement_cert(stake_pool.id(), 0);
        self.transaction_with_cert(&owners, certificate, ledger)
    }

    fn transaction_with_cert(
        &self,
        wallets: &[&Wallet],
        certificate: Certificate,
        test_ledger: &mut TestLedger,
    ) -> Fragment {
        TestTxCertBuilder::new(&test_ledger).make_transaction(wallets, &certificate)
    }
}
