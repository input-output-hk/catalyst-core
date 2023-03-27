#![allow(missing_docs)]

use crate::{
    data::{Registration, SignedRegistration, SlotNo},
    error::InvalidRegistration,
    verify::{filter_registrations, StakeKeyHash},
    DataProvider, SnapshotEntry,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use color_eyre::eyre::{eyre, Result};
use dashmap::DashMap;

use postgres::Client;

mod args;
pub use args::VotingPowerArgs;

/// Calculate voting power info by querying a db-sync instance
///
/// ```no_run
/// # use voting_tools_rs::{Db, VotingPowerArgs};
/// # fn connect() -> Db { unimplemented!() }
/// let db: Db = connect();  // get a database connection
/// let args = VotingPowerArgs::default();
/// let (valids, invalids) = voting_power(db, args);
///
/// // `valids` contains all successful registrations
/// // `invalids` contains failed registrations, with a reason:
/// for invalid in invalids {
///     println!("failed registration - reasons: ")
///     for error in invalid.error {
///         // ...
///     }
/// }
/// ```
///
/// Returns a tuple containing the successful snapshot entries, as well as any registrations which
/// failed verification in some way (along with some reason why they failed).
///
/// If provided, `min_slot` and `max_slot` can  be used to constrain the time period to query. If
/// `None` they default to:
///  - `min_slot`: `0`
///  - `max_slot`: `i64::MAX`
///
/// Together they form an inclusive range (i.e. blocks with values equal to `min_slot` or `max_slot` are included)
///
/// # Errors
///
/// Returns an error if either of `lower` or `upper` doesn't fit in an `i64`
pub fn voting_power(
    db: impl DataProvider,
    registration_client: Client,
    VotingPowerArgs {
        min_slot,
        max_slot,
        network_id,
        expected_voting_purpose,
    }: VotingPowerArgs,
) -> Result<(Vec<SnapshotEntry>, Vec<InvalidRegistration>)> {
    const ABS_MIN_SLOT: SlotNo = SlotNo(0);
    const ABS_MAX_SLOT: SlotNo = SlotNo(i64::MAX as u64);

    let min_slot = min_slot.unwrap_or(ABS_MIN_SLOT);
    let max_slot = max_slot.unwrap_or(ABS_MAX_SLOT);

    let (valids, invalids) =
        filter_registrations(min_slot, max_slot, registration_client, network_id).unwrap();

    let addrs = stake_addrs_hashes(&valids);

    let voting_powers = db.stake_values(&addrs);

    let snapshot = valids
        .into_iter()
        .map(|reg| convert_to_snapshot_entry(reg, &voting_powers))
        .collect::<Result<_, _>>()?;

    Ok((snapshot, invalids))
}

// returns hashes
fn stake_addrs_hashes(registrations: &[SignedRegistration]) -> Vec<StakeKeyHash> {
    let mut stake_keys = vec![];
    for r in registrations {
        stake_keys.push(r.stake_key_hash.clone());
    }
    stake_keys
}

fn convert_to_snapshot_entry(
    registration: SignedRegistration,
    voting_powers: &DashMap<StakeKeyHash, BigDecimal>,
) -> Result<SnapshotEntry> {
    let SignedRegistration {
        registration:
            Registration {
                voting_key,
                stake_key,
                rewards_address,
                voting_purpose,
                ..
            },
        tx_id,
        stake_key_hash,
        ..
    } = registration;

    let voting_power = voting_powers
        .get(&stake_key_hash)
        .ok_or_else(|| eyre!("no voting power available for stake key: {}", stake_key))?;

    let voting_power = voting_power.to_u128().unwrap_or(0);

    Ok(SnapshotEntry {
        voting_key,
        rewards_address,
        stake_key,
        voting_power,
        voting_purpose,
        tx_id,
    })
}
