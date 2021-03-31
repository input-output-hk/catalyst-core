use crate::cli::args::qr::IapyxQrCommandError;
use chain_core::mempack::ReadBuf;
use chain_core::mempack::Readable;
use chain_core::property::Deserialize;
use chain_impl_mockchain::block::Block;
use jormungandr_lib::interfaces::{Block0Configuration, Initial};
use regex::Regex;
use std::collections::HashMap;
use std::io::BufReader;
use std::ops::Range;
use std::path::Path;
use structopt::StructOpt;
use url::Url;

#[derive(StructOpt, Debug)]
pub struct WalletsCommand {
    #[structopt(long = "block0")]
    pub block0: String,
    #[structopt(long = "threshold")]
    pub threshold: u64,
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    Distribution,
}

impl WalletsCommand {
    pub fn exec(&self) -> Result<(), IapyxQrCommandError> {
        match self.command {
            Command::Distribution => self.calculate_wallet_distribution(),
        }
    }

    fn calculate_wallet_distribution(&self) -> Result<(), IapyxQrCommandError> {
        println!("Reading block0 from location {:?}...", &self.block0);

        let block = {
            if Path::new(&self.block0).exists() {
                let reader = std::fs::OpenOptions::new()
                    .create(false)
                    .write(false)
                    .read(true)
                    .append(false)
                    .open(&self.block0)?;
                let reader = BufReader::new(reader);
                Block::deserialize(reader)?
            } else if Url::parse(&self.block0).is_ok() {
                let response = reqwest::blocking::get(&self.block0)?;
                let block0_bytes = response.bytes()?.to_vec();
                Block::read(&mut ReadBuf::from(&block0_bytes))?
            } else {
                panic!(" block0 should be either path to filesystem or url ");
            }
        };
        let genesis = Block0Configuration::from_block(&block)?;

        let headers = vec![
            (0..self.threshold),
            (self.threshold..10_000),
            (10_000..20_000),
            (20_000..50_000),
            (50_000..100_000),
            (100_000..250_000),
            (250_000..500_000),
            (500_000..1_000_000),
            (1_000_000..5_000_000),
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
