use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::{
    data::{Registration, SignedRegistration, SlotNo, StakeKeyHex},
    validation::ValidationError,
    DataProvider, SnapshotEntry,
};
use bigdecimal::BigDecimal;
use chrono::Utc;
use color_eyre::eyre::{eyre, Result};
use itertools::Itertools;
use validity::{Valid, Validate};

#[cfg(test)]
mod tests;

/// Calculate voting power info by querying a db-sync instance
///
/// Invalid registrations are silently ignored (e.g. if they contain bad/null JSON metadata, if
/// they have invalid signatures, etc).
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
    min_slot: Option<SlotNo>,
    max_slot: Option<SlotNo>,
) -> Result<Vec<SnapshotEntry>> {
    let min_slot = min_slot.unwrap_or(SlotNo(0));
    let max_slot = max_slot.unwrap_or(SlotNo(u64::try_from(i64::MAX).unwrap()));

    let registrations = db.vote_registrations(min_slot, max_slot)?;

    let (valid_registrations, validation_errors): (Vec<_>, Vec<_>) = registrations
        .into_iter()
        .map(Validate::validate)
        .partition_result();

    show_error_warning(&validation_errors)?;

    let addrs = stake_addrs(&valid_registrations);
    let voting_powers = db.stake_values(&addrs)?;

    valid_registrations
        .into_iter()
        .map(|reg| convert_to_snapshot_entry(reg, &voting_powers))
        .collect()
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

/// If there are errors, we want to notify the user, but it's not really actionable, so we provide
/// the option to silence the error via env var
fn show_error_warning(errors: &[ValidationError]) -> Result<()> {
    let num_errs = errors.len();

    if num_errs == 0 || std::env::var("VOTING_TOOL_SUPPRESS_WARNINGS").unwrap() == "1" {
        return Ok(());
    }

    warn!("{num_errs} rows generated errors, set `VOTING_TOOL_SUPPRESS_WARNINGS=1 to suppress this warning");

    let path = error_log_file()?;
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);

    for e in errors {
        writeln!(&mut writer, "{e}")?;
    }

    warn!("error logs have been written to {}", path.to_string_lossy());

    Ok(())
}

fn error_log_file() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().expect("no home dir found to write logs");
    let error_dir = home_dir.join(".voting_tool_logs");
    std::fs::create_dir_all(&error_dir)?;

    let now = Utc::now();
    let log_file = error_dir.join(now.format("%Y-%m-%d--%H-%M-%S").to_string());

    Ok(log_file)
}
