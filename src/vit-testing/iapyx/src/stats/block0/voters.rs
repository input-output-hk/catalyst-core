use crate::cli::args::stats::IapyxStatsCommandError;
use chain_crypto::bech32::Bech32;
use jormungandr_automation::{jormungandr::JormungandrRest, testing::block0::get_block};
use jormungandr_lib::interfaces::Initial;

pub fn count_active_voters<S: Into<String>>(endpoint: S) -> Result<(), IapyxStatsCommandError> {
    let endpoint = endpoint.into();
    let block0_path = format!("{}/v0/block0", &endpoint);
    println!("Reading block0 from location {:?}...", block0_path);
    let genesis = get_block(block0_path)?;

    let rest_client = JormungandrRest::new(endpoint);
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
                    if state.counters()[0] > 0 {
                        stats.voted_at_least_once += 1;
                    }
                }
            }
        }
    }

    println!("{:?}", stats);
    Ok(())
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
