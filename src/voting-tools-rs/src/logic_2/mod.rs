use std::collections::HashMap;

use crate::{
    data::{Registration, SignedRegistration, SlotNo, StakeKeyHex},
    error::InvalidRegistration,
    validation::ValidationCtx,
    DataProvider, SnapshotEntry,
};
use bigdecimal::BigDecimal;
use color_eyre::eyre::{eyre, Result};
use itertools::Itertools;
use nonempty::nonempty;
use validity::{Failure, Valid, Validate};

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
    db: impl DataProvider,
    VotingPowerArgs {
        min_slot,
        max_slot,
        network_id,
        expected_voting_purpose,
    }: VotingPowerArgs,
) -> Result<(Vec<SnapshotEntry>, Vec<InvalidRegistration>)> {
    let min_slot = min_slot.unwrap_or(SlotNo(0));
    let max_slot = max_slot.unwrap_or(SlotNo(u64::try_from(i64::MAX).unwrap()));

    let validation_ctx = ValidationCtx {
        db: &db,
        network_id,
        expected_voting_purpose,
    };

    let validate = |reg: SignedRegistration| {
        reg.validate_with(validation_ctx.clone())
            .map_err(|Failure { value, error }| InvalidRegistration {
                registration: Some(value),
                errors: nonempty![error],
            })
    };

    let registrations = db.vote_registrations(min_slot, max_slot)?;

    let (valid_registrations, validation_errors): (Vec<_>, Vec<_>) =
        registrations.into_iter().map(validate).partition_result();

    let addrs = stake_addrs(&valid_registrations);
    let voting_powers = db.stake_values(&addrs)?;

    let snapshot = valid_registrations
        .into_iter()
        .map(|reg| convert_to_snapshot_entry(reg, &voting_powers))
        .collect::<Result<_, _>>()?;

    Ok((snapshot, validation_errors))
}

fn stake_addrs(registrations: &[Valid<SignedRegistration>]) -> Vec<StakeKeyHex> {
    registrations
        .iter()
        .map(|reg| &reg.registration.stake_key)
        .cloned()
        .collect()
}

fn convert_to_snapshot_entry(
    registration: Valid<SignedRegistration>,
    voting_powers: &HashMap<StakeKeyHex, BigDecimal>,
) -> Result<SnapshotEntry> {
    let SignedRegistration {
        registration:
            Registration {
                voting_power_source,
                stake_key,
                rewards_address,
                voting_purpose,
                ..
            },
        tx_id,
        ..
    } = registration.into_inner();

    let voting_power = voting_powers.get(&stake_key).ok_or_else(|| {
        eyre!(
            "no voting power available for stake key: {}",
            stake_key.to_hex()
        )
    })?;

    let voting_power = voting_power.clone();

    Ok(SnapshotEntry {
        voting_power_source,
        rewards_address,
        stake_key,
        voting_power,
        voting_purpose,
        tx_id,
    })
}
