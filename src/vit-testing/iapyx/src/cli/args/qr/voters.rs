use crate::cli::args::qr::IapyxQrCommandError;
use chain_core::mempack::ReadBuf;
use chain_core::mempack::Readable;
use chain_crypto::bech32::Bech32;
use chain_impl_mockchain::block::Block;
use jormungandr_lib::interfaces::{Block0Configuration, Initial};
use jormungandr_testing_utils::testing::node::JormungandrRest;
use structopt::StructOpt;
use url::Url;

#[derive(StructOpt, Debug)]
pub struct VotersCommand {
    #[structopt(long = "endpoint")]
    pub endpoint: String,
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    Count,
}

impl VotersCommand {
    pub fn exec(&self) -> Result<(), IapyxQrCommandError> {
        match self.command {
            Command::Count => self.count_active_voters(),
        }
    }

    fn count_active_voters(&self) -> Result<(), IapyxQrCommandError> {
        let block0_path = format!("{}/v0/block0", self.endpoint);
        println!("Reading block0 from location {:?}...", block0_path);
        let block = {
            if Url::parse(&block0_path).is_ok() {
                let response = reqwest::blocking::get(&block0_path)?;

                let block0_bytes = response.bytes()?.to_vec();
                Block::read(&mut ReadBuf::from(&block0_bytes))?
            } else {
                panic!("cannot obtain block0 for endpoint");
            }
        };
        let genesis = Block0Configuration::from_block(&block)?;
        let rest_client = JormungandrRest::new(self.endpoint.clone());
        let mut stats: Stats = Default::default();

        let mut total = 0;
        for initial in genesis.initial.iter() {
            if let Initial::Fund(initial_utxos) = initial {
                total += initial_utxos.len();
            }
        }

        println!("total: {}", total);
        for initial in genesis.initial.iter() {
            if let Initial::Fund(initial_utxos) = initial {
                for x in initial_utxos {
                    stats.total += 1;

                    let entry_address: chain_addr::Address = x.address.clone().into();
                    let pk = entry_address.public_key().unwrap().to_bech32_str();
                    println!("[{}/{}] Checking address state {}", stats.total, total, pk);
                    if let Ok(state) = rest_client.account_state_by_pk(&pk) {
                        stats.obtained_voting_power += 1;
                        if state.counter() > 0 {
                            stats.voted_at_least_once += 1;
                        }
                    }
                }
            }
        }

        println!("{:?}", stats);
        Ok(())
    }
}

#[derive(Debug)]
struct Stats {
    total: u32,
    obtained_voting_power: u32,
    voted_at_least_once: u32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            total: 0,
            obtained_voting_power: 0,
            voted_at_least_once: 0,
        }
    }
}
