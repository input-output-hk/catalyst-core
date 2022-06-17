use crate::rewards::voters::{account_hex_to_address, VoteCount};
use crate::stats::distribution::Stats;
use chain_addr::{Discrimination, Kind};
use chain_crypto::Ed25519;
use chain_impl_mockchain::vote::CommitteeId;
use jormungandr_automation::testing::block0::get_block;
use jormungandr_lib::interfaces::Address;
use jormungandr_lib::interfaces::Block0Configuration;
use jormungandr_lib::interfaces::Initial;
use jormungandr_lib::interfaces::InitialUTxO;
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

fn vote_counts_as_addresses(
    votes_count: VoteCount,
    genesis: &Block0Configuration,
) -> Vec<(InitialUTxO, u32)> {
    genesis
        .initial
        .iter()
        .filter_map(|initials| {
            if let Initial::Fund(funds) = initials {
                for utxo in funds {
                    if let Some((_, votes_count)) = votes_count.iter().find(|(address, _)| {
                        account_hex_to_address(
                            address.to_string(),
                            genesis.blockchain_configuration.discrimination,
                        )
                        .unwrap()
                            == utxo.address
                    }) {
                        return Some((utxo.clone(), *votes_count as u32));
                    }
                }
            }
            None
        })
        .collect()
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
    let initials = vote_counts_as_addresses(vote_count, &genesis);

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
