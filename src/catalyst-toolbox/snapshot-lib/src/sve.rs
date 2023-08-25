use std::collections::HashMap;

use chain_addr::Discrimination;
use jormungandr_lib::{
    crypto::account::Identifier,
    interfaces::{InitialUTxO, Value},
};

use crate::{
    registration::{Delegations, VotingRegistration},
    RawSnapshot,
};

pub struct Snapshot {
    inner: HashMap<Identifier, Vec<VotingRegistration>>,
}

impl Snapshot {
    pub fn new(raw_snapshot: RawSnapshot, min_stake_threshold: Value) -> (Self, usize) {
        let mut total_rejected_registrations: usize = 0;

        let inner = raw_snapshot
            .0
            .into_iter()
            .filter(|r| {
                if let Delegations::New(ds) = &r.delegations {
                    if ds.len() != 1 {
                        // eprintln!(
                        //     "Multiple Delegations unsupported for SVE:\n{}",
                        //     serde_json::to_string_pretty(r).unwrap()
                        // );

                        total_rejected_registrations += 1;
                        return false;
                    }
                }

                true
            })
            .filter(|r| r.voting_power >= min_stake_threshold)
            .fold(HashMap::<Identifier, Vec<_>>::new(), |mut acc, r| {
                let k = match &r.delegations {
                    Delegations::New(ds) => ds.first().unwrap().0.clone(),
                    Delegations::Legacy(id) => id.clone(),
                };

                acc.entry(k).or_default().push(r);
                acc
            });

        (Self { inner }, total_rejected_registrations)
    }

    pub fn to_block0_initials(
        &self,
        discrimination: Discrimination,
        lovelace: bool,
    ) -> Vec<InitialUTxO> {
        self.inner
            .iter()
            .map(|(vk, regs)| {
                let mut value: Value = regs
                    .iter()
                    .map(|reg| u64::from(reg.voting_power))
                    .sum::<u64>()
                    .into();

                //convert to ADA
                if !lovelace{
                    value = (u64::from(value)/1_000_000).into();
                }

                let address = chain_addr::Address(
                    discrimination,
                    chain_addr::Kind::Account(vk.to_inner().into()),
                )
                .into();

                InitialUTxO { address, value }
            })
            .collect()
    }
}
