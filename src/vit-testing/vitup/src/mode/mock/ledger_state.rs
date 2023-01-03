use chain_addr::Discrimination;
use chain_core::property::Block;
use chain_core::property::Fragment as _;
use chain_impl_mockchain::fee::LinearFee;
use chain_impl_mockchain::fragment::Fragment;
use chain_impl_mockchain::fragment::FragmentId;
use chain_impl_mockchain::ledger::{Error as LedgerError, Ledger};
use chain_impl_mockchain::testing::TestGen;
use chain_impl_mockchain::transaction::Transaction;
use chain_impl_mockchain::vote::VotePlanStatus;
use jormungandr_lib::interfaces::Block0Configuration;
use jormungandr_lib::interfaces::RejectedFragmentInfo;
use jormungandr_lib::interfaces::{BlockDate, SettingsDto};
use jormungandr_lib::interfaces::{FragmentLog, FragmentOrigin, FragmentStatus};
use jormungandr_lib::interfaces::{FragmentRejectionReason, FragmentsProcessingSummary};
use jormungandr_lib::time::SystemTime;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use thiserror::Error;
use thor::BlockDateGenerator;

#[derive(Copy, Clone, Debug)]
pub enum FragmentRecieveStrategy {
    Reject,
    Accept,
    Pending,
    None,
    //For cases when we want to implement mempool cleaning
    Forget,
}

impl Display for FragmentRecieveStrategy {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct LedgerState {
    fragment_strategy: FragmentRecieveStrategy,
    fragment_logs: Vec<FragmentLog>,
    received_fragments: Vec<Fragment>,
    ledger: Ledger,
    block0_configuration: Block0Configuration,
}

impl LedgerState {
    pub fn new(block0_configuration: Block0Configuration) -> Result<Self, Error> {
        let block = block0_configuration.to_block();

        Ok(Self {
            fragment_strategy: FragmentRecieveStrategy::None,
            fragment_logs: Vec::new(),
            received_fragments: Vec::new(),
            block0_configuration,
            ledger: Ledger::new(block.id(), block.fragments())?,
        })
    }

    pub fn message(&mut self, fragment: Fragment) -> FragmentId {
        self.received_fragments.push(fragment.clone());
        let fragment_id = fragment.id();
        let date = self.current_blockchain_age();
        let result = self.ledger.apply_fragment(&fragment, date.into());
        let mut fragment_log = FragmentLog::new(fragment.id(), FragmentOrigin::Rest);
        self.set_fragment_status(&mut fragment_log, self.fragment_strategy, result);
        if !(matches!(self.fragment_strategy, FragmentRecieveStrategy::Forget)) {
            self.fragment_logs.push(fragment_log);
        }
        fragment_id
    }

    pub fn batch_message(
        &mut self,
        fragments: Vec<Fragment>,
        fail_fast: bool,
    ) -> FragmentsProcessingSummary {
        let mut filtered_fragments = Vec::new();
        let mut rejected = Vec::new();
        let mut fragments = fragments.into_iter();

        for fragment in fragments.by_ref() {
            let id = fragment.id();
            if self
                .fragment_logs
                .iter()
                .any(|x| *x.fragment_id() == id.into())
            {
                rejected.push(RejectedFragmentInfo {
                    id,
                    reason: FragmentRejectionReason::FragmentAlreadyInLog,
                });
                continue;
            }

            if !is_fragment_valid(&fragment) {
                rejected.push(RejectedFragmentInfo {
                    id,
                    reason: FragmentRejectionReason::FragmentInvalid,
                });

                if fail_fast {
                    break;
                }

                continue;
            }
            filtered_fragments.push(fragment);
        }

        if fail_fast {
            for fragment in fragments.by_ref() {
                let id = fragment.id();
                rejected.push(RejectedFragmentInfo {
                    id,
                    reason: FragmentRejectionReason::PreviousFragmentInvalid,
                })
            }
        }

        let mut accepted = HashSet::new();

        for fragment in filtered_fragments {
            let id = fragment.id();
            self.message(fragment);
            accepted.insert(id);
        }

        let accepted = accepted.into_iter().collect();
        FragmentsProcessingSummary { accepted, rejected }
    }

    pub fn statuses(&self, ids: Vec<FragmentId>) -> HashMap<String, FragmentStatus> {
        self.fragment_logs
            .iter()
            .filter(|x| ids.contains(&(*x.fragment_id()).into_hash()))
            .map(|x| (x.fragment_id().to_string(), x.status().clone()))
            .collect()
    }

    pub fn set_fragment_strategy(&mut self, fragment_strategy: FragmentRecieveStrategy) {
        self.fragment_strategy = fragment_strategy;
    }

    pub fn accounts(&self) -> &chain_impl_mockchain::account::Ledger {
        self.ledger.accounts()
    }

    pub fn active_vote_plans(&self) -> Vec<VotePlanStatus> {
        self.ledger.active_vote_plans()
    }

    pub fn set_status_for_recent_fragment(
        &mut self,
        fragment_strategy: FragmentRecieveStrategy,
    ) -> Option<jormungandr_lib::crypto::hash::Hash> {
        let block_date = self.current_blockchain_age();
        if let Some(fragment_log) = self.fragment_logs.last_mut() {
            override_fragment_status(block_date, fragment_log, fragment_strategy);
            Some(*fragment_log.fragment_id())
        } else {
            None
        }
    }

    pub fn set_status_for_fragment_id(
        &mut self,
        fragment_id: String,
        fragment_strategy: FragmentRecieveStrategy,
    ) -> Result<(), Error> {
        let block_date = self.current_blockchain_age();
        let fragment_log = self
            .fragment_logs
            .iter_mut()
            .find(|x| x.fragment_id().to_string() == fragment_id)
            .ok_or(Error::CannotFindFragment(fragment_id))?;
        override_fragment_status(block_date, fragment_log, fragment_strategy);
        Ok(())
    }

    pub fn set_fragment_status(
        &mut self,
        fragment_log: &mut FragmentLog,
        fragment_strategy: FragmentRecieveStrategy,
        result: Result<Ledger, LedgerError>,
    ) {
        if let FragmentRecieveStrategy::None = fragment_strategy {
            match result {
                Ok(ledger) => {
                    self.ledger = ledger;
                    fragment_log.modify(FragmentStatus::InABlock {
                        date: self.current_blockchain_age(),
                        block: TestGen::hash().into(),
                    })
                }
                Err(error) => fragment_log.modify(FragmentStatus::Rejected {
                    reason: format!("{:?}", error),
                }),
            };
        } else {
            override_fragment_status(
                self.current_blockchain_age(),
                fragment_log,
                fragment_strategy,
            );
        }
    }

    pub fn fragment_logs(&self) -> Vec<FragmentLog> {
        self.fragment_logs.clone()
    }

    pub fn received_fragments(&self) -> Vec<Fragment> {
        self.received_fragments.clone()
    }

    pub fn curr_slot_start_time(&self) -> SystemTime {
        let blockchain_configuration = &self.block0_configuration.blockchain_configuration;

        let slot_duration: u8 = blockchain_configuration.slot_duration.into();
        let slots_per_epoch: u32 = blockchain_configuration.slots_per_epoch.into();
        let last_block_date = self.current_blockchain_age();
        let secs = last_block_date.epoch() * slot_duration as u32 * slots_per_epoch
            + slot_duration as u32 * last_block_date.slot();
        let block0_time: std::time::SystemTime =
            jormungandr_lib::time::SystemTime::from(blockchain_configuration.block0_date).into();
        block0_time
            .checked_add(std::time::Duration::from_secs(secs.into()))
            .unwrap()
            .into()
    }

    pub fn current_blockchain_age(&self) -> BlockDate {
        let blockchain_configuration = &self.block0_configuration.blockchain_configuration;

        let slot_duration: u8 = blockchain_configuration.slot_duration.into();
        let slots_per_epoch: u32 = blockchain_configuration.slots_per_epoch.into();
        BlockDateGenerator::current_blockchain_age(
            SystemTime::from(blockchain_configuration.block0_date),
            slots_per_epoch,
            slot_duration.into(),
        )
        .into()
    }

    pub fn absolute_slot_count(&self) -> u32 {
        let settings = self.settings();
        let block_date = self.current_blockchain_age();
        block_date.epoch() * settings.slots_per_epoch + block_date.slot()
    }

    pub fn settings(&self) -> SettingsDto {
        let params = self.ledger.settings();
        let slot_duration: u8 = self
            .block0_configuration
            .blockchain_configuration
            .slot_duration
            .into();

        SettingsDto {
            block0_hash: self.block0_hash().to_string(),
            block0_time: SystemTime::from_secs_since_epoch(
                self.block0_configuration
                    .blockchain_configuration
                    .block0_date
                    .to_secs(),
            ),
            discrimination: Discrimination::Production,
            curr_slot_start_time: Some(self.curr_slot_start_time()),
            consensus_version: self
                .block0_configuration
                .blockchain_configuration
                .block0_consensus
                .to_string(),
            fees: self.fees(),
            block_content_max_size: self
                .block0_configuration
                .blockchain_configuration
                .block_content_max_size
                .into(),
            epoch_stability_depth: self
                .block0_configuration
                .blockchain_configuration
                .epoch_stability_depth
                .into(),
            slot_duration: slot_duration as u64,
            slots_per_epoch: self
                .block0_configuration
                .blockchain_configuration
                .slots_per_epoch
                .into(),
            treasury_tax: params.treasury_params(),
            reward_params: params.reward_params(),
            tx_max_expiry_epochs: params.transaction_max_expiry_epochs,
        }
    }

    #[allow(dead_code)]
    pub fn expiry_date(&self) -> BlockDateGenerator {
        BlockDateGenerator::rolling_from_blockchain_config(
            &self.block0_configuration.blockchain_configuration,
            chain_impl_mockchain::block::BlockDate {
                epoch: 1,
                slot_id: 0,
            },
            false,
        )
    }

    pub fn block0_hash(&self) -> chain_impl_mockchain::key::Hash {
        self.block0_configuration.to_block().id()
    }

    pub fn fees(&self) -> LinearFee {
        self.block0_configuration
            .blockchain_configuration
            .linear_fees
            .clone()
    }
}

pub fn override_fragment_status(
    block_date: BlockDate,
    fragment_log: &mut FragmentLog,
    fragment_strategy: FragmentRecieveStrategy,
) {
    match fragment_strategy {
        FragmentRecieveStrategy::Pending => {
            fragment_log.modify(FragmentStatus::Pending);
        }
        FragmentRecieveStrategy::Accept => {
            fragment_log.modify(FragmentStatus::InABlock {
                date: block_date,
                block: TestGen::hash().into(),
            });
        }
        FragmentRecieveStrategy::Reject => {
            fragment_log.modify(FragmentStatus::Rejected {
                reason: "Force reject by mock".to_string(),
            });
        }
        _ => {}
    }
}

fn is_fragment_valid(fragment: &Fragment) -> bool {
    match fragment {
        // never valid in the pool, only acceptable in genesis
        Fragment::Initial(_) => false,
        Fragment::OldUtxoDeclaration(_) => false,
        // general transactions stuff
        Fragment::Evm(_) => false,
        Fragment::EvmMapping(ref tx) => is_transaction_valid(tx),
        Fragment::MintToken(ref tx) => is_transaction_valid(tx),
        Fragment::Transaction(ref tx) => is_transaction_valid(tx),
        Fragment::StakeDelegation(ref tx) => is_transaction_valid(tx),
        Fragment::OwnerStakeDelegation(ref tx) => is_transaction_valid(tx),
        Fragment::PoolRegistration(ref tx) => is_transaction_valid(tx),
        Fragment::PoolRetirement(ref tx) => is_transaction_valid(tx),
        Fragment::PoolUpdate(ref tx) => is_transaction_valid(tx),
        // vote stuff
        Fragment::UpdateProposal(ref tx) => is_transaction_valid(tx),
        Fragment::UpdateVote(ref tx) => is_transaction_valid(tx),
        Fragment::VotePlan(ref tx) => is_transaction_valid(tx),
        Fragment::VoteCast(ref tx) => is_transaction_valid(tx),
        Fragment::VoteTally(ref tx) => is_transaction_valid(tx),
    }
}

fn is_transaction_valid<E>(tx: &Transaction<E>) -> bool {
    tx.verify_possibly_balanced().is_ok()
}

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Error {
    #[error(transparent)]
    Ledger(#[from] chain_impl_mockchain::ledger::Error),
    #[error("cannot find fragment: {0}")]
    CannotFindFragment(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use jormungandr_automation::testing::configuration::Block0ConfigurationBuilder;
    use jormungandr_lib::interfaces::Initial;
    use jormungandr_lib::interfaces::InitialUTxO;
    use quickcheck_macros::quickcheck;
    use thor::FragmentBuilder;

    pub fn block0_configuration(initials: Vec<InitialUTxO>) -> Block0Configuration {
        Block0ConfigurationBuilder::default()
            .with_funds(vec![Initial::Fund(initials)])
            .with_some_consensus_leader()
            .build()
    }

    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for FragmentRecieveStrategy {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            match u8::arbitrary(g) % 3 {
                0 => FragmentRecieveStrategy::Pending,
                1 => FragmentRecieveStrategy::Accept,
                2 => FragmentRecieveStrategy::Reject,
                3 => FragmentRecieveStrategy::Forget,
                _ => unreachable!(),
            }
        }
    }

    #[test]
    pub fn message_fragment_id() {
        let alice = thor::Wallet::default();
        let bob = thor::Wallet::default();

        let mut ledger_state = LedgerState::new(block0_configuration(vec![
            alice.to_initial_fund(1_000),
            bob.to_initial_fund(1_000),
        ]))
        .unwrap();

        let fragment_builder = FragmentBuilder::new(
            &ledger_state.block0_hash().into(),
            &ledger_state.fees(),
            ledger_state.expiry_date().block_date(),
        );
        let fragment = fragment_builder
            .transaction(&alice, bob.address(), 1u64.into())
            .unwrap();

        assert_eq!(fragment.id(), ledger_state.message(fragment.clone()));
        assert!(ledger_state.received_fragments().contains(&fragment));
    }

    #[quickcheck]
    pub fn fragment_strategy_test(fragment_strategy: FragmentRecieveStrategy) {
        let alice = thor::Wallet::default();
        let bob = thor::Wallet::default();

        let mut ledger_state = LedgerState::new(block0_configuration(vec![
            alice.to_initial_fund(1_000),
            bob.to_initial_fund(1_000),
        ]))
        .unwrap();

        ledger_state.set_fragment_strategy(fragment_strategy);

        let fragment_builder = FragmentBuilder::new(
            &ledger_state.block0_hash().into(),
            &ledger_state.fees(),
            ledger_state.expiry_date().block_date(),
        );
        let fragment = fragment_builder
            .transaction(&alice, bob.address(), 1u64.into())
            .unwrap();

        assert_eq!(fragment.id(), ledger_state.message(fragment.clone()));
        assert_eq!(ledger_state.received_fragments().len(), 1);
        assert!(ledger_state.received_fragments().contains(&fragment));

        match fragment_strategy {
            FragmentRecieveStrategy::Pending => {
                assert_eq!(ledger_state.fragment_logs().len(), 1);
                let fragment_log = &ledger_state.fragment_logs()[0];
                assert_eq!(fragment_log.fragment_id().into_hash(), fragment.id());
                assert!(fragment_log.is_pending());
            }
            FragmentRecieveStrategy::Accept => {
                assert_eq!(ledger_state.fragment_logs().len(), 1);
                let fragment_log = &ledger_state.fragment_logs()[0];
                assert_eq!(fragment_log.fragment_id().into_hash(), fragment.id());
                assert!(fragment_log.is_in_a_block());
            }
            FragmentRecieveStrategy::Reject => {
                assert_eq!(ledger_state.fragment_logs().len(), 1);
                let fragment_log = &ledger_state.fragment_logs()[0];
                assert_eq!(fragment_log.fragment_id().into_hash(), fragment.id());
                assert!(fragment_log.is_rejected());
            }
            FragmentRecieveStrategy::Forget => {
                assert_eq!(ledger_state.received_fragments().len(), 0);
            }
            FragmentRecieveStrategy::None => {
                assert_eq!(ledger_state.received_fragments().len(), 1);
            }
        }
    }
}
