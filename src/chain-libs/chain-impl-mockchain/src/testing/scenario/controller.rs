use crate::{
    certificate::{ExternalProposalId, VoteCast, VotePlan, VoteTally},
    fee::LinearFee,
    key::Hash,
    ledger::Error as LedgerError,
    testing::{
        data::{StakePool, Wallet},
        ledger::TestLedger,
        scenario::template::VotePlanDef,
    },
    vote::{Choice, Payload},
};

#[cfg(test)]
use super::scenario_builder::{prepare_scenario, stake_pool, wallet};
use super::FragmentFactory;
#[cfg(test)]
use chain_addr::Discrimination;

use thiserror::Error;

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

    /*
    fn empty_context() -> HeaderContentEvalContext {
        HeaderContentEvalContext {
            block_date: BlockDate::first(),
            chain_length: ChainLength(0),
            nonce: None,
        }
    }
    */

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
        let fragment = self
            .fragment_factory
            .stake_pool_registration(funder, stake_pool);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn delegates(
        &self,
        from: &Wallet,
        stake_pool: &StakePool,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self.fragment_factory.delegation(from, stake_pool);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn delegates_different_funder(
        &self,
        funder: &Wallet,
        delegation: &Wallet,
        stake_pool: &StakePool,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self
            .fragment_factory
            .delegation_different_funder(funder, delegation, stake_pool);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn removes_delegation(
        &self,
        from: &Wallet,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self.fragment_factory.delegation_remove(from);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn delegates_to_many(
        &self,
        from: &Wallet,
        distribution: &[(&StakePool, u8)],
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self.fragment_factory.delegation_to_many(from, distribution);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn owner_delegates(
        &self,
        from: &Wallet,
        stake_pool: &StakePool,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self.fragment_factory.owner_delegation(from, stake_pool);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn retire(
        &self,
        owners: &[&Wallet],
        stake_pool: &StakePool,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self.fragment_factory.stake_pool_retire(owners, stake_pool);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn update(
        &self,
        stake_pool: &StakePool,
        update: StakePool,
        owners: Vec<&Wallet>,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let fragment = self
            .fragment_factory
            .stake_pool_update(owners, stake_pool, update);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn cast_vote(
        &self,
        owner: &Wallet,
        vote_plan_def: &VotePlanDef,
        id: &ExternalProposalId,
        choice: Choice,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let vote_plan: VotePlan = vote_plan_def.clone().into();
        let index = vote_plan
            .proposals()
            .iter()
            .enumerate()
            .find(|(_, x)| *x.external_id() == *id)
            .expect("cannot find proposal")
            .0 as u8;
        let vote_cast = VoteCast::new(vote_plan.to_id(), index, Payload::Public { choice });

        let fragment = self.fragment_factory.vote_cast(owner, vote_cast);
        test_ledger.apply_fragment(&fragment, test_ledger.date())
    }

    pub fn tally_vote(
        &self,
        owner: &Wallet,
        vote_plan_def: &VotePlanDef,
        test_ledger: &mut TestLedger,
    ) -> Result<(), LedgerError> {
        let vote_plan: VotePlan = vote_plan_def.clone().into();
        let vote_tally = VoteTally::new_public(vote_plan.to_id());

        let fragment = self.fragment_factory.vote_tally(owner, vote_tally);
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
                ConfigBuilder::new(0)
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
            .retire(&[&clarice], &stake_pool, &mut ledger)
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
