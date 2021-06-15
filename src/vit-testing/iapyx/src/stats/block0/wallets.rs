use crate::cli::args::stats::IapyxStatsCommandError;
use chain_core::mempack::ReadBuf;
use chain_core::mempack::Readable;
use chain_core::property::Deserialize;
use chain_impl_mockchain::block::Block;
use core::ops::Range;
use jormungandr_lib::interfaces::Block0Configuration;
use jormungandr_lib::interfaces::Initial;
use regex::Regex;
use std::collections::HashMap;
use std::io::BufReader;
use std::path::Path;
use url::Url;

pub fn calculate_wallet_distribution<S: Into<String>>(
    block0: S,
    threshold: u64,
) -> Result<(), IapyxStatsCommandError> {
    let block0 = block0.into();
    println!("Reading block0 from location {:?}...", &block0);

    let block = {
        if Path::new(&block0).exists() {
            let reader = std::fs::OpenOptions::new()
                .create(false)
                .write(false)
                .read(true)
                .append(false)
                .open(&block0)?;
            let reader = BufReader::new(reader);
            Block::deserialize(reader)?
        } else if Url::parse(&block0).is_ok() {
            let response = reqwest::blocking::get(&block0)?;
            let block0_bytes = response.bytes()?.to_vec();
            Block::read(&mut ReadBuf::from(&block0_bytes))?
        } else {
            panic!(" block0 should be either path to filesystem or url ");
        }
    };
    let genesis = Block0Configuration::from_block(&block)?;

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

    for initial in genesis.initial.iter() {
        if let Initial::Fund(initial_utxos) = initial {
            for x in initial_utxos {
                stats.add(x.value.into())
            }
        }
    }

    println!("Wallet distribution: \n{:#?}", stats);
    Ok(())
}

struct Stats {
    pub content: HashMap<Range<u64>, u32>,
}

impl Stats {
    pub fn new(header: Vec<Range<u64>>) -> Self {
        Self {
            content: header.into_iter().map(|range| (range, 0u32)).collect(),
        }
    }

    pub fn add(&mut self, value: u64) {
        for (range, count) in self.content.iter_mut() {
            if range.contains(&value) {
                *count += 1;
                return;
            }
        }
    }

    pub fn sum(&self) -> u32 {
        self.content.values().sum()
    }
}

use std::fmt;

impl fmt::Debug for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut keys = self.content.keys().cloned().collect::<Vec<Range<u64>>>();
        keys.sort_by_key(|x| x.start);

        for key in keys {
            let start = format_big_number(key.start.to_string());
            let end = format_big_number(key.end.to_string());
            f.write_str(&format!(
                "{} .. {} -> {} \n",
                start, end, self.content[&key]
            ))?;
        }
        f.write_str(&format!("Total -> {} ", self.sum()))?;
        Ok(())
    }
}

fn format_big_number<S: Into<String>>(number: S) -> String {
    let mld = Regex::new(r"000000000$").unwrap();
    let mln = Regex::new(r"000000$").unwrap();
    let k = Regex::new(r"000$").unwrap();

    let mut output = mld.replace_all(&number.into(), " MLD").to_string();
    output = mln.replace_all(&output, " M").to_string();
    k.replace_all(&output, " k").to_string()
}
