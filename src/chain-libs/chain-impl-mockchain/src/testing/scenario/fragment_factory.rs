#[cfg(feature = "evm")]
use crate::certificate::EvmMapping;
#[cfg(feature = "evm")]
use crate::evm::EvmTransaction;
use crate::{
    accounting::account::{DelegationRatio, DelegationType},
    certificate::{
        Certificate, MintToken, PoolId, PoolUpdate, UpdateProposal, UpdateVote, VoteCast, VotePlan,
        VoteTally,
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
        WitnessMode,
    },
    value::Value,
};

#[derive(Clone, Debug)]
pub struct FragmentFactory {
    pub block0_hash: Hash,
    pub fee: LinearFee,
    pub witness_mode: WitnessMode,
}

impl FragmentFactory {
    pub fn from_ledger(test_ledger: &TestLedger) -> Self {
        Self::new(test_ledger.block0_hash, test_ledger.fee())
    }

    pub fn new(block0_hash: Hash, fee: LinearFee) -> Self {
        Self {
            block0_hash,
            fee,
            witness_mode: Default::default(),
        }
    }

    pub fn witness_mode(mut self, witness_mode: WitnessMode) -> Self {
        self.witness_mode = witness_mode;
        self
    }

    pub fn transaction(
        &self,
        from: &Wallet,
        to: &Wallet,
        test_ledger: &mut TestLedger,
        funds: u64,
    ) -> Fragment {
        TestTxBuilder::new(test_ledger.block0_hash)
            .witness_mode(self.witness_mode)
            .move_funds(
                test_ledger,
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
        let pools_ratio_sum: u8 = distribution.iter().map(|(_st, ratio)| *ratio).sum();
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

    pub fn vote_tally(
        &self,
        valid_until: BlockDate,
        owner: &Wallet,
        vote_tally: VoteTally,
    ) -> Fragment {
        self.transaction_with_cert(valid_until, Some(owner), &vote_tally.into())
    }

    pub fn update_proposal(
        &self,
        valid_until: BlockDate,
        owner: &Wallet,
        signer: &Wallet,
        update_proposal: UpdateProposal,
    ) -> Fragment {
        TestTxCertBuilder::new(self.block0_hash, self.fee.clone())
            .make_transaction_different_signers(
                valid_until,
                owner,
                vec![signer],
                &update_proposal.into(),
                self.witness_mode,
            )
    }

    pub fn update_vote(
        &self,
        valid_until: BlockDate,
        owner: &Wallet,
        signer: &Wallet,
        update_vote: UpdateVote,
    ) -> Fragment {
        TestTxCertBuilder::new(self.block0_hash, self.fee.clone())
            .make_transaction_different_signers(
                valid_until,
                owner,
                vec![signer],
                &update_vote.into(),
                self.witness_mode,
            )
    }

    pub fn mint_token(
        &self,
        valid_until: BlockDate,
        owner: &Wallet,
        min_token: MintToken,
    ) -> Fragment {
        self.transaction_with_cert(valid_until, Some(owner), &min_token.into())
    }

    fn transaction_with_cert<'a>(
        &self,
        valid_until: BlockDate,
        wallets: impl IntoIterator<Item = &'a Wallet>,
        certificate: &Certificate,
    ) -> Fragment {
        TestTxCertBuilder::new(self.block0_hash, self.fee.clone()).make_transaction(
            valid_until,
            wallets,
            certificate,
            self.witness_mode,
        )
    }

    #[cfg(feature = "evm")]
    pub fn evm_mapping(
        &self,
        valid_until: BlockDate,
        owner: &Wallet,
        evm_mapping: EvmMapping,
    ) -> Fragment {
        self.transaction_with_cert(valid_until, Some(owner), &evm_mapping.into())
    }

    #[cfg(feature = "evm")]
    pub fn evm_transaction(&self, evm_transaction: EvmTransaction) -> Fragment {
        Fragment::Evm(evm_transaction)
    }
}
