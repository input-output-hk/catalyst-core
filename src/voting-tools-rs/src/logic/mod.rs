#![allow(missing_docs)]

use std::thread;

use crate::{
    data::{Registration, SignedRegistration, SlotNo},
    db::queries::staked_utxo_ada::staked_utxo_ada,
    error::InvalidRegistration,
    verify::{filter_registrations, StakeKeyHash},
    SnapshotEntry,
};

use crate::verify::Unregistered;
use color_eyre::eyre::Result;
use dashmap::DashMap;

use postgres::Client;

mod args;
pub use args::VotingPowerArgs;

/// Calculate voting power info by querying a db-sync instance
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
    mut db_client_stakes: Client,
    db_client_registrations: Client,
    VotingPowerArgs {
        min_slot,
        max_slot,
        network_id,
        expected_voting_purpose: _,
        cip_36_multidelegations,
    }: VotingPowerArgs,
) -> Result<(Vec<SnapshotEntry>, Vec<InvalidRegistration>, Unregistered)> {
    const ABS_MIN_SLOT: SlotNo = SlotNo(0);
    const ABS_MAX_SLOT: SlotNo = SlotNo(i64::MAX as u64);

    let min_slot = min_slot.unwrap_or(ABS_MIN_SLOT);
    let max_slot = max_slot.unwrap_or(ABS_MAX_SLOT);

    info!("starting stakes job");
    let stakes = thread::spawn(move || {
        staked_utxo_ada(i64::try_from(max_slot.0).unwrap(), &mut db_client_stakes).unwrap()
    });

    info!("starting registrations job");
    let registrations = thread::spawn(move || {
        filter_registrations(
            min_slot,
            max_slot,
            db_client_registrations,
            network_id,
            cip_36_multidelegations,
        )
        .unwrap()
    });

    let (valids, invalids) = registrations.join().unwrap();
    info!("finished processing registrations");

    // UTXOs for all possible Stake Addresses
    let staked_ada_records = stakes.join().unwrap();
    info!("finished processing stakes");

    let snapshot = valids
        .into_iter()
        .map(|reg| convert_to_snapshot_entry(reg, &staked_ada_records))
        .collect::<Result<_, _>>()?;

    Ok((snapshot, invalids, staked_ada_records))
}

fn convert_to_snapshot_entry(
    registration: SignedRegistration,
    stakes: &DashMap<StakeKeyHash, u128>,
) -> Result<SnapshotEntry> {
    let SignedRegistration {
        registration:
            Registration {
                voting_key,
                stake_key,
                rewards_address,
                nonce,
                voting_purpose,
            },
        tx_id,
        stake_key_hash,
        ..
    } = registration;

    // look up stake key hash of valid registration in stakes map to obtain staked ada associated with the key
    let voting_power = if let Some(voting_power) = stakes.get(&stake_key_hash) {
        *voting_power
    } else {
        // Registrations with no staked ada. No UTXO's.
        0
    };

    // remove registered, what is left (difference) are stake addresses that are not registered
    stakes.remove(&stake_key_hash);

    Ok(SnapshotEntry {
        voting_key,
        rewards_address,
        stake_key,
        voting_power,
        voting_purpose,
        tx_id,
        nonce: nonce.0,
    })
}
