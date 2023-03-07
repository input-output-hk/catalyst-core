use crate::{
    data::{Registration, SignedRegistration, SlotNo, StakeKeyHex},
    error::InvalidRegistration,
    validation::ValidationCtx,
    DataProvider, SnapshotEntry,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use color_eyre::eyre::{eyre, Result};
use dashmap::DashMap;
use itertools::Itertools;
use nonempty::nonempty;
use validity::{Failure, Valid, Validate};

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

    let ctx = ValidationCtx {
        network_id,
        expected_voting_purpose,
        validate_network_id: false,
        validate_key_type: false,
        ..Default::default()
    };

    let validate = |reg: SignedRegistration| {
        reg.validate_with(ctx)
            .map_err(|Failure { value, error }| InvalidRegistration {
                registration: Some(value),
                errors: nonempty![error],
            })
    };

    let registrations = db.vote_registrations(min_slot, max_slot)?;
    info!("found {} registrations", registrations.len());

    let (valid_registrations, validation_errors): (Vec<_>, Vec<_>) =
        registrations.into_iter().map(validate).partition_result();

    let addrs = stake_addrs(&valid_registrations);

    let voting_powers = db.stake_values(&addrs);

    let snapshot = valid_registrations
        .into_iter()
        .map(|reg| convert_to_snapshot_entry(reg, &voting_powers))
        .collect::<Result<_, _>>()?;

    Ok((snapshot, validation_errors))
}

fn stake_addrs(registrations: &[Valid<SignedRegistration>]) -> Vec<StakeKeyHex> {
    registrations
        .iter()
        .map(|reg| reg.registration.stake_key.clone())
        .collect()
}

fn convert_to_snapshot_entry(
    registration: Valid<SignedRegistration>,
    voting_powers: &DashMap<StakeKeyHex, BigDecimal>,
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

    let voting_power = voting_powers
        .get(&stake_key)
        .ok_or_else(|| eyre!("no voting power available for stake key: {}", stake_key))?;

    let voting_power = voting_power.to_u128().unwrap_or(0);

    Ok(SnapshotEntry {
        voting_power_source,
        rewards_address,
        stake_key,
        voting_power,
        voting_purpose,
        tx_id,
    })
}
