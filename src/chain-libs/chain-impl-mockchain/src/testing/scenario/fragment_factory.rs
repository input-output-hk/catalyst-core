use crate::{
    accounting::account::{DelegationRatio, DelegationType},
    certificate::{
        Certificate, EncryptedVoteTally, PoolId, PoolUpdate, VoteCast, VotePlan, VoteTally,
    },
    date::BlockDate,
    fee::LinearFee,
    fragment::Fragment,
    key::Hash,
    testing::{
        builders::{
            build_no_stake_delegation, build_owner_stake_delegation,
            build_owner_stake_full_delegation, build_stake_delegation_cert,
            build_stake_pool_registration_cert, build_stake_pool_retirement_cert,
            build_stake_pool_update_cert, TestTxBuilder, TestTxCertBuilder,
        },
        data::{StakePool, Wallet},
        ledger::TestLedger,
    },
    value::Value,
};

#[derive(Clone, Debug)]
pub struct FragmentFactory {
    pub block0_hash: Hash,
    pub fee: LinearFee,
}

impl FragmentFactory {
    pub fn from_ledger(test_ledger: &TestLedger) -> Self {
        Self::new(test_ledger.block0_hash, test_ledger.fee())
    }

    pub fn new(block0_hash: Hash, fee: LinearFee) -> Self {
        Self { block0_hash, fee }
    }

    pub fn transaction(
        &self,
        from: &Wallet,
        to: &Wallet,
        mut test_ledger: &mut TestLedger,
        funds: u64,
    ) -> Fragment {
        TestTxBuilder::new(test_ledger.block0_hash)
            .move_funds(
                &mut test_ledger,
                &from.as_account(),
                &to.as_account(),
                Value(funds),
            )
            .get_fragment()
    }

    pub fn stake_pool_registration(
        &self,
        valid_until: BlockDate,
        funder: &Wallet,
        stake_pool: &StakePool,
    ) -> Fragment {
        let cert = build_stake_pool_registration_cert(&stake_pool.info());
        self.transaction_with_cert(valid_until, Some(funder), &cert)
    }

    pub fn delegation(
        &self,
        valid_until: BlockDate,
        from: &Wallet,
        stake_pool: &StakePool,
    ) -> Fragment {
        let cert = build_stake_delegation_cert(&stake_pool.info(), &from.as_account_data());
        self.transaction_with_cert(valid_until, Some(from), &cert)
    }

    pub fn delegation_different_funder(
        &self,
        valid_until: BlockDate,
        funder: &Wallet,
        delegation: &Wallet,
        stake_pool: &StakePool,
    ) -> Fragment {
        let cert = build_stake_delegation_cert(&stake_pool.info(), &delegation.as_account_data());
        self.transaction_with_cert(valid_until, Some(funder), &cert)
    }

    pub fn delegation_remove(&self, valid_until: BlockDate, from: &Wallet) -> Fragment {
        let cert = build_no_stake_delegation();
        self.transaction_with_cert(valid_until, Some(from), &cert)
    }

    pub fn delegation_to_many(
        &self,
        valid_until: BlockDate,
        from: &Wallet,
        distribution: &[(&StakePool, u8)],
    ) -> Fragment {
        let pools_ratio_sum: u8 = distribution.iter().map(|(_st, ratio)| *ratio as u8).sum();
        let pools: Vec<(PoolId, u8)> = distribution
            .iter()
            .map(|(st, ratio)| (st.info().to_id(), *ratio))
            .collect();

        let delegation_ratio = DelegationRatio::new(pools_ratio_sum, pools);
        let delegation_type = DelegationType::Ratio(delegation_ratio.unwrap());
        let cert = build_owner_stake_delegation(delegation_type);
        self.transaction_with_cert(valid_until, Some(from), &cert)
    }

    pub fn owner_delegation(
        &self,
        valid_until: BlockDate,
        from: &Wallet,
        stake_pool: &StakePool,
    ) -> Fragment {
        let cert = build_owner_stake_full_delegation(stake_pool.id());
        self.transaction_with_cert(valid_until, Some(from), &cert)
    }

    pub fn stake_pool_retire<'a>(
        &self,
        valid_until: BlockDate,
        owners: impl IntoIterator<Item = &'a Wallet>,
        stake_pool: &StakePool,
    ) -> Fragment {
        let certificate = build_stake_pool_retirement_cert(stake_pool.id(), 0);
        self.transaction_with_cert(valid_until, owners, &certificate)
    }

    pub fn stake_pool_update<'a>(
        &self,
        valid_until: BlockDate,
        owners: impl IntoIterator<Item = &'a Wallet>,
        stake_pool: &StakePool,
        update: StakePool,
    ) -> Fragment {
        let pool_update = PoolUpdate {
            pool_id: stake_pool.id(),
            last_pool_reg_hash: stake_pool.info().to_id(),
            new_pool_reg: update.info(),
        };
        let certificate = build_stake_pool_update_cert(&pool_update);
        self.transaction_with_cert(valid_until, owners, &certificate)
    }

    pub fn vote_plan(
        &self,
        valid_until: BlockDate,
        owner: &Wallet,
        vote_plan: VotePlan,
    ) -> Fragment {
        self.transaction_with_cert(valid_until, Some(owner), &vote_plan.into())
    }

    pub fn vote_cast(
        &self,
        valid_until: BlockDate,
        owner: &Wallet,
        vote_cast: VoteCast,
    ) -> Fragment {
        self.transaction_with_cert(valid_until, Some(owner), &vote_cast.into())
    }

    pub fn vote_encrypted_tally(
        &self,
        valid_until: BlockDate,
        owner: &Wallet,
        encrypted_tally: EncryptedVoteTally,
    ) -> Fragment {
        self.transaction_with_cert(valid_until, Some(owner), &encrypted_tally.into())
    }

    pub fn vote_tally(
        &self,
        valid_until: BlockDate,
        owner: &Wallet,
        vote_tally: VoteTally,
    ) -> Fragment {
        self.transaction_with_cert(valid_until, Some(owner), &vote_tally.into())
    }

    fn transaction_with_cert<'a>(
        &self,
        valid_until: BlockDate,
        wallets: impl IntoIterator<Item = &'a Wallet>,
        certificate: &Certificate,
    ) -> Fragment {
        TestTxCertBuilder::new(self.block0_hash, self.fee).make_transaction(
            valid_until,
            wallets,
            certificate,
        )
    }
}
