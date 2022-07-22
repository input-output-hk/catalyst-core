use crate::rewards::voters::VoteCount;
use crate::stats::distribution::Stats;
use chain_addr::{Discrimination, Kind};
use chain_crypto::Ed25519;
use chain_impl_mockchain::vote::CommitteeId;
use jormungandr_automation::testing::block0::get_block;
use jormungandr_lib::{
    crypto::account::Identifier,
    interfaces::{Address, Block0Configuration, Initial, InitialUTxO},
};
use std::path::Path;

fn blacklist_addresses(genesis: &Block0Configuration) -> Vec<Address> {
    let discrimination = genesis.blockchain_configuration.discrimination;

    genesis
        .blockchain_configuration
        .committees
        .iter()
        .cloned()
        .map(|x| {
            let committee_id: CommitteeId = x.into();
            let public: chain_crypto::PublicKey<Ed25519> =
                chain_crypto::PublicKey::from_binary(committee_id.as_ref()).unwrap();

            Address(
                if discrimination == Discrimination::Production {
                    "ca".to_string()
                } else {
                    "ta".to_string()
                },
                chain_addr::Address(discrimination, Kind::Account(public)),
            )
        })
        .collect()
}

fn vote_counts_as_utxo(
    votes_count: VoteCount,
    genesis: &Block0Configuration,
) -> Vec<(InitialUTxO, u32)> {
    genesis
        .initial
        .iter()
        .filter_map(|initials| {
            if let Initial::Fund(funds) = initials {
                for utxo in funds {
                    if let Some(vote_count) = votes_count.get(&addr_to_hex(&utxo.address)) {
                        return Some((utxo.clone(), *vote_count as u32));
                    }
                }
            }
            None
        })
        .collect()
}

pub fn addr_to_hex(address: &Address) -> String {
    match &address.1 .1 {
        Kind::Account(pk) => {
            let id: Identifier = pk.clone().into();
            id.to_hex()
        }
        _ => unimplemented!(),
    }
}

pub fn calculate_active_wallet_distribution<S: Into<String>, P: AsRef<Path>>(
    stats: Stats,
    block0: S,
    votes_count_path: P,
    support_lovelace: bool,
    update_fn: impl Fn(&mut Stats, u64, u64),
) -> Result<Stats, crate::stats::Error> {
    let block0 = block0.into();
    let genesis = get_block(block0)?;

    let vote_count: VoteCount = serde_json::from_reader(jcli_lib::utils::io::open_file_read(
        &Some(votes_count_path.as_ref()),
    )?)?;

    let blacklist = blacklist_addresses(&genesis);
    let initials = vote_counts_as_utxo(vote_count, &genesis);

    calculate_wallet_distribution_from_initials_utxo(
        stats,
        initials,
        blacklist,
        support_lovelace,
        update_fn,
    )
}

pub fn calculate_wallet_distribution<S: Into<String>>(
    block0: S,
    stats: Stats,
    support_lovelace: bool,
    update_fn: impl Fn(&mut Stats, u64, u64),
) -> Result<Stats, crate::stats::Error> {
    let block0 = block0.into();
    let genesis = get_block(block0)?;
    let blacklist = blacklist_addresses(&genesis);

    calculate_wallet_distribution_from_initials(
        stats,
        genesis.initial,
        blacklist,
        support_lovelace,
        update_fn,
    )
}

pub fn calculate_wallet_distribution_from_initials(
    stats: Stats,
    initials: Vec<Initial>,
    blacklist: Vec<Address>,
    support_lovelace: bool,
    update_fn: impl Fn(&mut Stats, u64, u64),
) -> Result<Stats, crate::stats::Error> {
    let mut utxos = vec![];
    for initial in initials.into_iter() {
        if let Initial::Fund(initial_utxos) = initial {
            for x in initial_utxos.into_iter() {
                utxos.push(x)
            }
        }
    }

    calculate_wallet_distribution_from_initials_utxo(
        stats,
        utxos.iter().cloned().map(|x| (x, 1u32)).collect(),
        blacklist,
        support_lovelace,
        update_fn,
    )
}

pub fn calculate_wallet_distribution_from_initials_utxo(
    mut stats: Stats,
    initials: Vec<(InitialUTxO, u32)>,
    blacklist: Vec<Address>,
    support_lovelace: bool,
    update_fn: impl Fn(&mut Stats, u64, u64),
) -> Result<Stats, crate::stats::Error> {
    for (x, weight) in initials {
        if !blacklist.contains(&x.address) {
            let mut value: u64 = x.value.into();
            if support_lovelace {
                value /= 1_000_000;
            }
            update_fn(&mut stats, value, weight.into());
        }
    }
    Ok(stats)
}
