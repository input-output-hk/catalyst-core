use crate::cli::args::stats::IapyxStatsCommandError;
use crate::stats::distribution::Stats;
use chain_addr::{Discrimination, Kind};
use chain_crypto::Ed25519;
use chain_impl_mockchain::vote::CommitteeId;
use jormungandr_lib::interfaces::Address;
use jormungandr_lib::interfaces::Initial;
use jormungandr_testing_utils::testing::block0::get_block;

pub fn calculate_wallet_distribution<S: Into<String>>(
    block0: S,
    threshold: u64,
    support_lovelace: bool,
) -> Result<Stats, IapyxStatsCommandError> {
    let block0 = block0.into();
    println!("Reading block0 from location {:?}...", &block0);
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

            Address(
                "ca".to_string(),
                chain_addr::Address(Discrimination::Production, Kind::Account(public)),
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
) -> Result<Stats, IapyxStatsCommandError> {
    let mut stats = Stats::new(threshold);
    for initial in initials.iter() {
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
