use crate::stats::distribution::Stats;
use chain_addr::{Discrimination, Kind};
use chain_crypto::Ed25519;
use chain_impl_mockchain::vote::CommitteeId;
use jormungandr_automation::testing::block0::get_block;
use jormungandr_lib::interfaces::Address;
use jormungandr_lib::interfaces::Initial;
use jormungandr_lib::interfaces::InitialUTxO;

pub fn calculate_wallet_distribution<S: Into<String>>(
    block0: S,
    threshold: u64,
    support_lovelace: bool,
) -> Result<Stats, crate::stats::Error> {
    let block0 = block0.into();
    let genesis = get_block(block0)?;

    #[allow(clippy::needless_collect)]
    let blacklist: Vec<Address> = genesis
        .blockchain_configuration
        .committees
        .iter()
        .cloned()
        .map(|x| {
            let committee_id: CommitteeId = x.into();
            let public: chain_crypto::PublicKey<Ed25519> =
                chain_crypto::PublicKey::from_binary(committee_id.as_ref()).unwrap();
            let discrimination = genesis.blockchain_configuration.discrimination;

            Address(
                if discrimination == Discrimination::Production {
                    "ca".to_string()
                } else {
                    "ta".to_string()
                },
                chain_addr::Address(discrimination, Kind::Account(public)),
            )
        })
        .collect();

    calculate_wallet_distribution_from_initials(
        genesis.initial,
        blacklist,
        threshold,
        support_lovelace,
    )
}

pub fn calculate_wallet_distribution_from_initials(
    initials: Vec<Initial>,
    blacklist: Vec<Address>,
    threshold: u64,
    support_lovelace: bool,
) -> Result<Stats, crate::stats::Error> {
    let mut utxos = vec![];
    for initial in initials.into_iter() {
        if let Initial::Fund(initial_utxos) = initial {
            for x in initial_utxos.into_iter() {
                utxos.push(x)
            }
        }
    }

    calculate_wallet_distribution_from_initials_utxo(utxos, blacklist, threshold, support_lovelace)
}

pub fn calculate_wallet_distribution_from_initials_utxo(
    initials: Vec<InitialUTxO>,
    blacklist: Vec<Address>,
    threshold: u64,
    support_lovelace: bool,
) -> Result<Stats, crate::stats::Error> {
    let mut stats = Stats::new(threshold);

    for x in initials {
        if !blacklist.contains(&x.address) {
            let mut value: u64 = x.value.into();
            if support_lovelace {
                value /= 1_000_000;
            }
            stats.add(value);
        }
    }

    Ok(stats)
}
