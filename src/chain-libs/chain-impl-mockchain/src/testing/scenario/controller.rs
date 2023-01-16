#[cfg(feature = "evm")]
use crate::certificate::EvmMapping;
#[cfg(feature = "evm")]
use crate::evm::EvmTransaction;
use crate::{
    certificate::{
        DecryptedPrivateTally, ExternalProposalId, MintToken, Proposal, UpdateProposal, UpdateVote,
        VoteCast, VotePlan, VoteTally,
    },
    fee::LinearFee,
    key::Hash,
    ledger::Error as LedgerError,
    testing::{
        data::{StakePool, Wallet},
        ledger::TestLedger,
        scenario::template::VotePlanDef,
        VoteTestGen,
    },
    vote::{Choice, Payload, PayloadType},
};

#[cfg(test)]
use super::scenario_builder::{prepare_scenario, stake_pool, wallet};
use super::FragmentFactory;
use crate::fragment::FragmentId;
#[cfg(test)]
use chain_addr::Discrimination;
use chain_core::property::Fragment as _;

use rand_core::{CryptoRng, RngCore};
use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ControllerError {
    #[error("cannot find wallet with alias {alias}")]
    UnknownWallet { alias: String },
    #[error("cannot find stake pool with alias {alias}")]
    UnknownStakePool { alias: String },
    #[error("cannot find vote plan with alias {alias}")]
    UnknownVotePlan { alias: String },
    #[error("cannot find vote proposal with alias {id}")]
    UnknownVoteProposal { id: ExternalProposalId },
}

pub struct Controller {
    pub block0_hash: Hash,
    pub declared_wallets: Vec<Wallet>,
    pub declared_stake_pools: Vec<StakePool>,
    pub declared_vote_plans: Vec<VotePlanDef>,
    fragment_factory: FragmentFactory,
}

impl Controller {
    pub fn new(
        block0_hash: Hash,
        fee: LinearFee,
        declared_wallets: Vec<Wallet>,
        declared_stake_pools: Vec<StakePool>,
        declared_vote_plans: Vec<VotePlanDef>,
    ) -> Self {
        Controller {
            block0_hash,
            declared_wallets,
            declared_stake_pools,
            declared_vote_plans,
            fragment_factory: FragmentFactory::new(block0_hash, fee),
        }
    }

    pub fn wallets(&self) -> Vec<Wallet> {
        self.declared_wallets.clone()
    }

    pub fn wallet(&self, alias: &str) -> Result<Wallet, ControllerError> {
        self.declared_wallets
            .iter()
            .cloned()
            .find(|x| x.alias() == alias)
            .ok_or(ControllerError::UnknownWallet {
                alias: alias.to_owned(),
            })
    }

    pub fn vote_plan(&self, alias: &str) -> Result<VotePlanDef, ControllerError> {
        self.declared_vote_plans
            .iter()
            .cloned()
            .find(|x| x.alias() == alias)
            .ok_or(ControllerError::UnknownVotePlan {
                alias: alias.to_owned(),
            })
    }

    pub fn initial_stake_pools(&self) -> Vec<StakePool> {
        self.declared_stake_pools.clone()
    }

    pub fn stake_pool(&self, alias: &str) -> Result<StakePool, ControllerError> {
        self.declared_stake_pools
            .iter()
            .cloned()
            .find(|x| x.alias() == alias)
            .ok_or(ControllerError::UnknownStakePool {
                alias: alias.to_owned(),
            })
    }

    pub fn fragment_factory(&self) -> FragmentFactory {
        self.fragment_factory.clone()
    }

    pub fn transfer_funds(
        &self,
        from: &Wallet,
        to: &Wallet,
        test_ledger: &mut TestLedger,
        funds: u64,
    ) -> Result<(), LedgerError> {
        let transaction = self
            .fragment_factory
            .transaction(from, to, test_ledger, funds);
        test_ledger.apply_transaction(transaction)
    }

    pub fn register(
        &self,
        funder: &Wallet,
        stake_pool: &StakePool,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment =
            self.fragment_factory
                .stake_pool_registration(test_ledger.date(), funder, stake_pool);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn delegates(
        &self,
        from: &Wallet,
        stake_pool: &StakePool,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self
            .fragment_factory
            .delegation(test_ledger.date(), from, stake_pool);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn delegates_different_funder(
        &self,
        funder: &Wallet,
        delegation: &Wallet,
        stake_pool: &StakePool,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self.fragment_factory.delegation_different_funder(
            test_ledger.date(),
            funder,
            delegation,
            stake_pool,
        );
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn removes_delegation(
        &self,
        from: &Wallet,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self
            .fragment_factory
            .delegation_remove(test_ledger.date(), from);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn delegates_to_many(
        &self,
        from: &Wallet,
        distribution: &[(&StakePool, u8)],
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment =
            self.fragment_factory
                .delegation_to_many(test_ledger.date(), from, distribution);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn owner_delegates(
        &self,
        from: &Wallet,
        stake_pool: &StakePool,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self
            .fragment_factory
            .owner_delegation(test_ledger.date(), from, stake_pool);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn retire<'a>(
        &'a self,
        owners: impl IntoIterator<Item = &'a Wallet>,
        stake_pool: &'a StakePool,
        test_ledger: &'a mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment =
            self.fragment_factory
                .stake_pool_retire(test_ledger.date(), owners, stake_pool);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn update<'a>(
        &'a self,
        stake_pool: &'a StakePool,
        update: StakePool,
        owners: impl IntoIterator<Item = &'a Wallet>,
        test_ledger: &'a mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment =
            self.fragment_factory
                .stake_pool_update(test_ledger.date(), owners, stake_pool, update);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn cast_vote_public(
        &self,
        owner: &Wallet,
        vote_plan_def: &VotePlanDef,
        id: &ExternalProposalId,
        choice: Choice,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        self.cast_vote(
            owner,
            vote_plan_def,
            id,
            test_ledger,
            |vote_plan, _proposal| match vote_plan.payload_type() {
                PayloadType::Public => Payload::Public { choice },
                PayloadType::Private => panic!("this is a private vote plan"),
            },
        )
    }

    pub fn cast_vote_private<R>(
        &self,
        owner: &Wallet,
        vote_plan_def: &VotePlanDef,
        id: &ExternalProposalId,
        choice: Choice,
        test_ledger: &mut TestLedger,
        rng: &mut R,
    ) -> Result<(), LedgerError>
    where
        R: RngCore + CryptoRng,
    {
        self.cast_vote(
            owner,
            vote_plan_def,
            id,
            test_ledger,
            |vote_plan, proposal| match vote_plan.payload_type() {
                PayloadType::Public => panic!("this is a public vote plan"),
                PayloadType::Private => {
                    VoteTestGen::private_vote_cast_payload_for(vote_plan, proposal, choice, rng)
                }
            },
        )
    }

    fn cast_vote<F>(
        &self,
        owner: &Wallet,
        vote_plan_def: &VotePlanDef,
        id: &ExternalProposalId,
        test_ledger: &mut TestLedger,
        mut payload_producer: F,
    ) -> Result<(), LedgerError>
    where
        F: FnMut(&VotePlan, &Proposal) -> Payload,
    {
        let vote_plan: VotePlan = vote_plan_def.clone().into();
        let (index, proposal) = vote_plan
            .proposals()
            .iter()
            .enumerate()
            .find(|(_, x)| *x.external_id() == *id)
            .map(|(index, proposal)| (index as u8, proposal))
            .expect("cannot find proposal");
        let payload = payload_producer(&vote_plan, proposal);
        let vote_cast = VoteCast::new(vote_plan.to_id(), index, payload);
        let fragment = self
            .fragment_factory
            .vote_cast(test_ledger.date(), owner, vote_cast);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn tally_vote_public(
        &self,
        owner: &Wallet,
        vote_plan_def: &VotePlanDef,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let vote_plan: VotePlan = vote_plan_def.clone().into();
        let vote_tally = VoteTally::new_public(vote_plan.to_id());

        let fragment = self
            .fragment_factory
            .vote_tally(test_ledger.date(), owner, vote_tally);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn tally_vote_private(
        &self,
        owner: &Wallet,
        vote_plan_def: &VotePlanDef,
        decrypted_tally: DecryptedPrivateTally,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let vote_plan: VotePlan = vote_plan_def.clone().into();
        let vote_tally = VoteTally::new_private(vote_plan.to_id(), decrypted_tally);

        let fragment = self
            .fragment_factory
            .vote_tally(test_ledger.date(), owner, vote_tally);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn update_proposal(
        &self,
        owner: &Wallet,
        update_proposal: UpdateProposal,
        test_ledger: &mut TestLedger,
    ) -> Result<FragmentId, LedgerError> {
        let fragment = self.fragment_factory.update_proposal(
            test_ledger.date(),
            owner,
            owner,
            update_proposal,
        );
        test_ledger.apply_fragment(&fragment, test_ledger.date())?;
        Ok(fragment.id())
    }

    pub fn update_vote(
        &self,
        owner: &Wallet,
        update_vote: UpdateVote,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment =
            self.fragment_factory
                .update_vote(test_ledger.date(), owner, owner, update_vote);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn mint_token(
        &self,
        owner: &Wallet,
        mint_token: MintToken,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self
            .fragment_factory
            .mint_token(test_ledger.date(), owner, mint_token);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    #[cfg(feature = "evm")]
    pub fn evm_mapping(
        &self,
        owner: &Wallet,
        evm_mapping: EvmMapping,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self
            .fragment_factory
            .evm_mapping(test_ledger.date(), owner, evm_mapping);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    #[cfg(feature = "evm")]
    pub fn evm_transaction(
        &self,
        evm_transaction: EvmTransaction,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self.fragment_factory.evm_transaction(evm_transaction);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        fee::LinearFee,
        stake::Stake,
        testing::{ledger::ConfigBuilder, verifiers::LedgerStateVerifier},
        value::Value,
    };

    #[test]
    pub fn build_scenario_example() {
        let (mut ledger, controller) = prepare_scenario()
            .with_config(
                ConfigBuilder::new()
                    .with_discrimination(Discrimination::Test)
                    .with_fee(LinearFee::new(1, 1, 1)),
            )
            .with_initials(vec![
                wallet("Alice").with(1_000).delegates_to("stake_pool"),
                wallet("Bob").with(1_000),
                wallet("Clarice").with(1_000).owns("stake_pool"),
            ])
            .with_stake_pools(vec![
                stake_pool("stake_pool").with_permissions_threshold(1u8)
            ])
            .build()
            .unwrap();
        let mut alice = controller.wallet("Alice").unwrap();
        let mut bob = controller.wallet("Bob").unwrap();
        let mut clarice = controller.wallet("Clarice").unwrap();
        let stake_pool = controller.stake_pool("stake_pool").unwrap();

        controller
            .transfer_funds(&alice, &bob, &mut ledger, 100)
            .unwrap();
        alice.confirm_transaction();
        controller
            .delegates(&bob, &stake_pool, &mut ledger)
            .unwrap();
        bob.confirm_transaction();
        controller
            .retire(Some(&clarice), &stake_pool, &mut ledger)
            .unwrap();
        clarice.confirm_transaction();
        // unassigned = clarice - fee (becaue thus clarise is an onwer of the stake she did not delegates any stakes)
        // dangling = bob and alice funds (minus fees for transactions and certs)
        // total pool = 0, because stake pool was retired

        LedgerStateVerifier::new(ledger.into())
            .distribution()
            .unassigned_is(Stake::from_value(Value(997)))
            .and()
            .dangling_is(Stake::from_value(Value(1994)))
            .and()
            .pools_total_stake_is(Stake::zero());
    }
}
