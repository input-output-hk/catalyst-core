use super::governance::Governance;
use super::ledger::{Error, Ledger, LedgerStaticParameters};
use super::pots::{self, Pots};
use super::LeadersParticipationRecord;
use crate::certificate::{VotePlan, VotePlanId};
use crate::chaintypes::ChainLength;
use crate::config::ConfigParam;
use crate::date::BlockDate;
use crate::key::Hash;
use crate::stake::PoolsState;
use crate::vote::{VotePlanLedger, VotePlanManager};
use crate::{account, legacy, multisig, setting, update, utxo};
use chain_addr::Address;
use chain_time::TimeEra;
use std::sync::Arc;

pub enum Entry<'a> {
    Globals(Globals),
    Pot(pots::Entry),
    Utxo(utxo::Entry<'a, Address>),
    OldUtxo(utxo::Entry<'a, legacy::OldAddress>),
    Account(
        (
            &'a account::Identifier,
            &'a crate::accounting::account::AccountState<()>,
        ),
    ),
    ConfigParam(ConfigParam),
    UpdateProposal(
        (
            &'a crate::update::UpdateProposalId,
            &'a crate::update::UpdateProposalState,
        ),
    ),
    MultisigAccount(
        (
            &'a crate::multisig::Identifier,
            &'a crate::accounting::account::AccountState<()>,
        ),
    ),
    MultisigDeclaration(
        (
            &'a crate::multisig::Identifier,
            &'a crate::multisig::Declaration,
        ),
    ),
    StakePool((&'a crate::certificate::PoolId, &'a crate::stake::PoolState)),
    LeaderParticipation((&'a crate::certificate::PoolId, &'a u32)),
    VotePlan(&'a VotePlan),
}

#[derive(Clone)]
pub enum EntryOwned {
    Globals(Globals),
    Pot(pots::Entry),
    Utxo(utxo::EntryOwned<Address>),
    OldUtxo(utxo::EntryOwned<legacy::OldAddress>),
    Account(
        (
            account::Identifier,
            crate::accounting::account::AccountState<()>,
        ),
    ),
    ConfigParam(ConfigParam),
    UpdateProposal(
        (
            crate::update::UpdateProposalId,
            crate::update::UpdateProposalState,
        ),
    ),
    MultisigAccount(
        (
            crate::multisig::Identifier,
            crate::accounting::account::AccountState<()>,
        ),
    ),
    MultisigDeclaration((crate::multisig::Identifier, crate::multisig::Declaration)),
    StakePool((crate::certificate::PoolId, crate::stake::PoolState)),
    LeaderParticipation((crate::certificate::PoolId, u32)),
    VotePlan(VotePlan),
    StopEntry,
}

impl EntryOwned {
    pub fn to_entry(&self) -> Option<Entry> {
        match self {
            EntryOwned::Globals(globals) => Some(Entry::Globals(globals.clone())),
            EntryOwned::Pot(entry) => Some(Entry::Pot(*entry)),
            EntryOwned::Utxo(entry) => {
                let utxo_entry = utxo::Entry {
                    fragment_id: entry.fragment_id,
                    output_index: entry.output_index,
                    output: &entry.output,
                };
                Some(Entry::Utxo(utxo_entry))
            }
            EntryOwned::OldUtxo(entry) => {
                let old_utxo_entry = utxo::Entry {
                    fragment_id: entry.fragment_id,
                    output_index: entry.output_index,
                    output: &entry.output,
                };
                Some(Entry::OldUtxo(old_utxo_entry))
            }
            EntryOwned::Account((identifier, account_state)) => {
                Some(Entry::Account((identifier, account_state)))
            }
            EntryOwned::ConfigParam(config_param) => Some(Entry::ConfigParam(config_param.clone())),
            EntryOwned::UpdateProposal((proposal_id, proposal_state)) => {
                Some(Entry::UpdateProposal((proposal_id, proposal_state)))
            }
            EntryOwned::MultisigAccount((identifier, account_state)) => {
                Some(Entry::MultisigAccount((identifier, account_state)))
            }
            EntryOwned::MultisigDeclaration((identifier, account_state)) => {
                Some(Entry::MultisigDeclaration((identifier, account_state)))
            }
            EntryOwned::StakePool((pool_id, pool_state)) => {
                Some(Entry::StakePool((pool_id, pool_state)))
            }
            EntryOwned::LeaderParticipation((pool_id, participation)) => {
                Some(Entry::LeaderParticipation((pool_id, participation)))
            }
            EntryOwned::VotePlan(vote_plan) => Some(Entry::VotePlan(vote_plan)),
            EntryOwned::StopEntry => None,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Globals {
    pub date: BlockDate,
    pub chain_length: ChainLength,
    pub static_params: LedgerStaticParameters,
    pub era: TimeEra,
}

enum IterState<'a> {
    Initial,
    Utxo(utxo::Iter<'a, Address>),
    OldUtxo(utxo::Iter<'a, legacy::OldAddress>),
    Accounts(crate::accounting::account::Iter<'a, account::Identifier, ()>),
    ConfigParams(Vec<ConfigParam>),
    UpdateProposals(
        std::collections::btree_map::Iter<
            'a,
            crate::update::UpdateProposalId,
            crate::update::UpdateProposalState,
        >,
    ),
    MultisigAccounts(crate::accounting::account::Iter<'a, crate::multisig::Identifier, ()>),
    MultisigDeclarations(
        imhamt::HamtIter<'a, crate::multisig::Identifier, crate::multisig::Declaration>,
    ),
    StakePools(imhamt::HamtIter<'a, crate::certificate::PoolId, crate::stake::PoolState>),
    Pots(pots::Entries<'a>),
    LeaderParticipations(imhamt::HamtIter<'a, crate::certificate::PoolId, u32>),
    VotePlan(imhamt::HamtIter<'a, VotePlanId, VotePlanManager>),
    Done,
}

pub struct LedgerIterator<'a> {
    ledger: &'a Ledger,
    state: IterState<'a>,
}

impl<'a> Iterator for LedgerIterator<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            IterState::Initial => {
                self.state = IterState::Utxo(self.ledger.utxos.iter());
                Some(Entry::Globals(Globals {
                    date: self.ledger.date,
                    chain_length: self.ledger.chain_length,
                    static_params: (*self.ledger.static_params).clone(),
                    era: self.ledger.era.clone(),
                }))
            }
            IterState::Utxo(iter) => match iter.next() {
                None => {
                    self.state = IterState::OldUtxo(self.ledger.oldutxos.iter());
                    self.next()
                }
                Some(x) => Some(Entry::Utxo(x)),
            },
            IterState::OldUtxo(iter) => match iter.next() {
                None => {
                    self.state = IterState::Accounts(self.ledger.accounts.iter());
                    self.next()
                }
                Some(x) => Some(Entry::OldUtxo(x)),
            },
            IterState::Accounts(iter) => match iter.next() {
                None => {
                    self.state = IterState::ConfigParams(self.ledger.settings.to_config_params().0);
                    self.next()
                }
                Some(x) => Some(Entry::Account(x)),
            },
            IterState::ConfigParams(params) => {
                if let Some(param) = params.pop() {
                    Some(Entry::ConfigParam(param))
                } else {
                    self.state = IterState::UpdateProposals(self.ledger.updates.proposals.iter());
                    self.next()
                }
            }
            IterState::UpdateProposals(iter) => match iter.next() {
                None => {
                    self.state = IterState::MultisigAccounts(self.ledger.multisig.iter_accounts());
                    self.next()
                }
                Some(x) => Some(Entry::UpdateProposal(x)),
            },
            IterState::MultisigAccounts(iter) => match iter.next() {
                None => {
                    self.state =
                        IterState::MultisigDeclarations(self.ledger.multisig.iter_declarations());
                    self.next()
                }
                Some(x) => Some(Entry::MultisigAccount(x)),
            },
            IterState::MultisigDeclarations(iter) => match iter.next() {
                None => {
                    self.state = IterState::StakePools(self.ledger.delegation.stake_pools.iter());
                    self.next()
                }
                Some(x) => Some(Entry::MultisigDeclaration(x)),
            },
            IterState::StakePools(iter) => match iter.next() {
                None => {
                    self.state = IterState::Pots(self.ledger.pots.entries());
                    self.next()
                }
                Some(x) => Some(Entry::StakePool(x)),
            },
            IterState::Pots(iter) => match iter.next() {
                None => {
                    self.state = IterState::LeaderParticipations(self.ledger.leaders_log.iter());
                    self.next()
                }
                Some(x) => Some(Entry::Pot(x)),
            },
            IterState::LeaderParticipations(iter) => match iter.next() {
                None => {
                    self.state = IterState::VotePlan(self.ledger.votes.plans.iter());
                    self.next()
                }
                Some(x) => Some(Entry::LeaderParticipation(x)),
            },
            IterState::VotePlan(iter) => match iter.next() {
                None => {
                    self.state = IterState::Done;
                    self.next()
                }
                Some((_, plan_manager)) => Some(Entry::VotePlan(plan_manager.plan())),
            },
            IterState::Done => None,
        }
    }
}

impl Ledger {
    pub fn iter(&self) -> LedgerIterator<'_> {
        LedgerIterator {
            ledger: self,
            state: IterState::Initial,
        }
    }
}

impl<'a> std::iter::FromIterator<Entry<'a>> for Result<Ledger, Error> {
    fn from_iter<I: IntoIterator<Item = Entry<'a>>>(iter: I) -> Self {
        use std::collections::HashMap;
        let mut utxos: HashMap<Hash, Vec<_>> = HashMap::new();
        let mut oldutxos: HashMap<Hash, Vec<_>> = HashMap::new();
        let mut accounts = vec![];
        let mut config_params = crate::fragment::ConfigParams::new();
        let mut updates = update::UpdateState::new();
        let mut multisig_accounts = vec![];
        let mut multisig_declarations = vec![];
        let delegation = PoolsState::new();
        let mut globals = None;
        let mut pots = Pots::zero();
        let mut leaders_log = LeadersParticipationRecord::new();
        // TODO: votes don't have their entry
        let mut votes = VotePlanLedger::new();
        let governance = Governance::default();

        for entry in iter {
            match entry {
                Entry::Globals(globals2) => {
                    globals = Some(globals2);
                    // FIXME: check duplicate
                }
                Entry::Utxo(entry) => {
                    utxos
                        .entry(entry.fragment_id)
                        .or_default()
                        .push((entry.output_index, entry.output.clone()));
                }
                Entry::OldUtxo(entry) => {
                    oldutxos
                        .entry(entry.fragment_id)
                        .or_default()
                        .push((entry.output_index, entry.output.clone()));
                }
                Entry::Account((account_id, account_state)) => {
                    accounts.push((account_id.clone(), account_state.clone()));
                }
                Entry::ConfigParam(param) => {
                    config_params.push(param.clone());
                }
                Entry::UpdateProposal((proposal_id, proposal_state)) => {
                    updates
                        .proposals
                        .insert(*proposal_id, proposal_state.clone());
                }
                Entry::MultisigAccount((account_id, account_state)) => {
                    multisig_accounts.push((account_id.clone(), account_state.clone()));
                }
                Entry::MultisigDeclaration((id, decl)) => {
                    multisig_declarations.push((id.clone(), decl.clone()));
                }
                Entry::StakePool((pool_id, pool_state)) => {
                    delegation
                        .stake_pools
                        .insert(pool_id.clone(), pool_state.clone())
                        .unwrap();
                }
                Entry::Pot(ent) => pots.set_from_entry(&ent),
                Entry::LeaderParticipation((pool_id, pool_participation)) => leaders_log
                    .set_for(pool_id.clone(), *pool_participation)
                    .unwrap(),
                Entry::VotePlan(vote_plan) => {
                    // TODO: don't use default
                    votes.plans = votes
                        .plans
                        .insert(
                            vote_plan.to_id(),
                            VotePlanManager::new(vote_plan.clone(), Default::default()),
                        )
                        .unwrap();
                }
            }
        }

        let globals = globals.ok_or(Error::IncompleteLedger)?;

        Ok(Ledger {
            utxos: utxos.into_iter().collect(),
            oldutxos: oldutxos.into_iter().collect(),
            accounts: accounts.into_iter().collect(),
            settings: setting::Settings::new().apply(&config_params)?,
            updates,
            multisig: multisig::Ledger::restore(multisig_accounts, multisig_declarations),
            delegation,
            static_params: Arc::new(globals.static_params),
            date: globals.date,
            chain_length: globals.chain_length,
            era: globals.era,
            pots,
            leaders_log,
            votes,
            governance,
        })
    }
}

#[cfg(any(test))]
mod tests {
    use super::*;
    use crate::{
        ledger::{Entry, Ledger},
        testing::{ConfigBuilder, LedgerBuilder},
        value::Value,
    };

    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Globals {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Globals {
                date: Arbitrary::arbitrary(g),
                chain_length: Arbitrary::arbitrary(g),
                static_params: Arbitrary::arbitrary(g),
                era: Arbitrary::arbitrary(g),
            }
        }
    }

    fn print_from_iter(ledger: &Ledger) {
        for item in ledger.iter() {
            match item {
                Entry::Globals(globals) => {
                    println!(
                    "Globals date={} length={} block0_hash={} start_time={:?} discr={} kes_update_speed={}",
                    globals.date,
                    globals.chain_length,
                    globals.static_params.block0_initial_hash,
                    globals.static_params.block0_start_time,
                    globals.static_params.discrimination,
                    globals.static_params.kes_update_speed,
                );
                }
                Entry::Utxo(entry) => {
                    println!(
                        "Utxo {} {} {}",
                        entry.fragment_id, entry.output_index, entry.output
                    );
                }
                Entry::OldUtxo(entry) => {
                    println!(
                        "OldUtxo {} {} {}",
                        entry.fragment_id, entry.output_index, entry.output
                    );
                }
                Entry::Account((id, state)) => {
                    println!(
                        "Account {} {} {:?} {}",
                        id,
                        u32::from(state.counter),
                        state.delegation,
                        state.value,
                    );
                }
                Entry::ConfigParam(param) => {
                    println!(
                        "ConfigParam {:?} {:?}",
                        crate::config::Tag::from(&param),
                        param,
                    );
                }
                Entry::UpdateProposal((id, state)) => {
                    println!(
                        "UpdateProposal {} {:?} {} {:?}",
                        id, state.proposal, state.proposal_date, state.votes
                    );
                }
                Entry::MultisigAccount((id, state)) => {
                    println!(
                        "MultisigAccount {} {} {:?} {}",
                        id,
                        u32::from(state.counter),
                        state.delegation,
                        state.value,
                    );
                }
                Entry::MultisigDeclaration((id, decl)) => {
                    println!(
                        "MultisigDeclaration {} {} {}",
                        id,
                        decl.threshold(),
                        decl.total(),
                    );
                }
                Entry::StakePool((id, state)) => {
                    let info = state.registration.as_ref();
                    println!(
                        "StakePool {} {} {:?} {:?}",
                        id, info.serial, info.owners, info.keys,
                    );
                }
                Entry::Pot(entry) => {
                    println!("Pot {:?}", entry);
                }
                Entry::LeaderParticipation((pool_id, pool_record)) => {
                    println!("LeaderParticipation {} {}", pool_id, pool_record);
                }
                Entry::VotePlan(plan) => {
                    println!("VotePlan {}", plan.to_id());
                }
            }
        }
    }

    #[test]
    pub fn iterate() {
        let testledger = LedgerBuilder::from_config(ConfigBuilder::new(0))
            .faucet_value(Value(42000))
            .build()
            .expect("cannot build test ledger");

        let ledger = testledger.ledger;

        let ledger2: Result<Ledger, _> = ledger.iter().collect();
        let ledger2 = ledger2.unwrap();

        print_from_iter(&ledger);

        assert!(ledger == ledger2)
    }
}
