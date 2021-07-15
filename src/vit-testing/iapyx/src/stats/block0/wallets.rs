use crate::cli::args::stats::IapyxStatsCommandError;
use chain_addr::{Discrimination, Kind};
use chain_crypto::Ed25519;
use chain_impl_mockchain::vote::CommitteeId;
use core::ops::Range;
use jormungandr_lib::interfaces::Address;
use jormungandr_lib::interfaces::Initial;
use jormungandr_testing_utils::testing::block0::get_block;
use regex::Regex;
use std::collections::HashMap;

pub fn calculate_wallet_distribution<S: Into<String>>(
    block0: S,
    threshold: u64,
    support_lovelace: bool,
) -> Result<Stats, IapyxStatsCommandError> {
    let block0 = block0.into();
    println!("Reading block0 from location {:?}...", &block0);
    let genesis = get_block(block0)?;

    let headers = vec![
        (0..threshold),
        (threshold..10_000),
        (10_000..20_000),
        (20_000..50_000),
        (50_000..100_000),
        (100_000..250_000),
        (250_000..500_000),
        (500_000..1_000_000),
        (1_000_000..5_000_000),
        (5_000_000..10_000_000),
        (10_000_000..25_000_000),
        (25_000_000..50_000_000),
        (50_000_000..32_000_000_000),
    ];

    let mut stats = Stats::new(headers);

    let blacklist: Vec<Address> = genesis
        .blockchain_configuration
        .committees
        .iter()
        .cloned()
        .map(|x| {
            let committee_id: CommitteeId = x.into();
            let public: chain_crypto::PublicKey<Ed25519> =
                chain_crypto::PublicKey::from_binary(committee_id.as_ref()).unwrap();

            Address(
                "ca".to_string(),
                chain_addr::Address(Discrimination::Production, Kind::Account(public)),
            )
        })
        .collect();

    for initial in genesis.initial.iter() {
        if let Initial::Fund(initial_utxos) = initial {
            for x in initial_utxos {
                if !blacklist.contains(&x.address) {
                    let mut value: u64 = x.value.into();
                    if support_lovelace {
                        value /= 1_000_000;
                    }
                    stats.add(value);
                }
            }
        }
    }
    Ok(stats)
}

pub struct Record {
    pub count: u32,
    pub total: u64,
}

impl Default for Record {
    fn default() -> Self {
        Record { count: 0, total: 0 }
    }
}

pub struct Stats {
    pub content: HashMap<Range<u64>, Record>,
}

impl Stats {
    pub fn new(header: Vec<Range<u64>>) -> Self {
        Self {
            content: header
                .into_iter()
                .map(|range| (range, Default::default()))
                .collect(),
        }
    }

    pub fn add(&mut self, value: u64) {
        for (range, record) in self.content.iter_mut() {
            if range.contains(&value) {
                record.count += 1;
                record.total += value;
                return;
            }
        }
    }

    pub fn print_count_per_level(&self) {
        let mut keys = self.content.keys().cloned().collect::<Vec<Range<u64>>>();
        keys.sort_by_key(|x| x.start);

        for key in keys {
            let start = format_big_number(key.start.to_string());
            let end = format_big_number(key.end.to_string());
            println!("{} .. {} -> {} ", start, end, self.content[&key].count);
        }
        println!(
            "Total -> {} ",
            self.content.values().map(|x| x.count).sum::<u32>()
        );
    }

    pub fn print_ada_per_level(&self) {
        let mut keys = self.content.keys().cloned().collect::<Vec<Range<u64>>>();
        keys.sort_by_key(|x| x.start);

        for key in keys {
            let start = format_big_number(key.start.to_string());
            let end = format_big_number(key.end.to_string());
            println!(
                "{} .. {} -> {} ",
                start,
                end,
                format_big_number(self.content[&key].total.to_string())
            );
        }
        println!(
            "Total -> {} ",
            self.content.values().map(|x| x.total).sum::<u64>()
        );
    }
}

fn format_big_number<S: Into<String>>(number: S) -> String {
    #[allow(clippy::trivial_regex)]
    let mld = Regex::new(r"000000000$").unwrap();
    #[allow(clippy::trivial_regex)]
    let mln = Regex::new(r"000000$").unwrap();
    #[allow(clippy::trivial_regex)]
    let k = Regex::new(r"000$").unwrap();

    let mut output = mld.replace_all(&number.into(), " MLD").to_string();
    output = mln.replace_all(&output, " M").to_string();
    k.replace_all(&output, " k").to_string()
}
