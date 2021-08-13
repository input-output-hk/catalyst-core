use super::NulPtr;
use crate::{Error, Result};
use chain_impl_mockchain::{config, fee, header::HeaderId};
use std::{convert::TryInto, num::NonZeroU64};
use wallet::Settings;

/// Linear fee using the basic affine formula
/// `COEFFICIENT * bytes(COUNT(tx.inputs) + COUNT(tx.outputs)) + CONSTANT + CERTIFICATE*COUNT(certificates)`.
#[repr(C)]
#[derive(Default)]
pub struct LinearFee {
    pub constant: u64,
    pub coefficient: u64,
    pub certificate: u64,
    pub per_certificate_fees: PerCertificateFee,
    pub per_vote_certificate_fees: PerVoteCertificateFee,
}

#[repr(C)]
#[derive(Default)]
pub struct PerCertificateFee {
    pub certificate_pool_registration: u64,
    pub certificate_stake_delegation: u64,
    pub certificate_owner_stake_delegation: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct PerVoteCertificateFee {
    pub certificate_vote_plan: u64,
    pub certificate_vote_cast: u64,
}

#[repr(C)]
pub enum Discrimination {
    Production = 0, // just for consistency, it's not like it matters
    Test,
}

pub type Epoch = u32;
pub type Slot = u64;

#[repr(C)]
pub struct TimeEra {
    pub epoch_start: Epoch,
    pub slot_start: Slot,
    pub slots_per_epoch: u32,
}

impl From<TimeEra> for chain_time::TimeEra {
    fn from(te: TimeEra) -> Self {
        chain_time::TimeEra::new(
            te.slot_start.into(),
            chain_time::Epoch(te.epoch_start),
            te.slots_per_epoch,
        )
    }
}

#[repr(C)]
pub struct SettingsInit {
    pub fees: LinearFee,
    pub discrimination: Discrimination,
    /// block_0_initial_hash is assumed to point to 32 bytes of readable memory
    pub block0_initial_hash: *const u8,
    /// Unix timestamp of the genesis block.
    /// Provides an anchor to compute block dates from calendar date/time.
    pub block0_date: u64,
    pub slot_duration: u8,
    pub time_era: TimeEra,
    pub transaction_max_expiry_epochs: u8,
}

/// # Safety
///
/// settings_out must point to valid writable memory
pub unsafe fn settings_new(settings: SettingsInit, settings_out: *mut *mut Settings) -> Result {
    let SettingsInit {
        fees,
        discrimination,
        block0_initial_hash,
        block0_date,
        slot_duration,
        time_era,
        transaction_max_expiry_epochs,
    } = settings;

    let settings_out = non_null_mut!(settings_out);

    let discrimination = match discrimination {
        Discrimination::Production => chain_addr::Discrimination::Production,
        Discrimination::Test => chain_addr::Discrimination::Test,
    };

    let block0_initial_hash =
        HeaderId::from_bytes(non_null_array!(block0_initial_hash, 32).try_into().unwrap());

    let linear_fee = fee::LinearFee {
        constant: fees.constant,
        coefficient: fees.coefficient,
        certificate: fees.certificate,
        per_certificate_fees: fee::PerCertificateFee {
            certificate_pool_registration: NonZeroU64::new(
                fees.per_certificate_fees.certificate_pool_registration,
            ),
            certificate_stake_delegation: NonZeroU64::new(
                fees.per_certificate_fees.certificate_stake_delegation,
            ),
            certificate_owner_stake_delegation: NonZeroU64::new(
                fees.per_certificate_fees.certificate_owner_stake_delegation,
            ),
        },
        per_vote_certificate_fees: fee::PerVoteCertificateFee {
            certificate_vote_plan: NonZeroU64::new(
                fees.per_vote_certificate_fees.certificate_vote_plan,
            ),
            certificate_vote_cast: NonZeroU64::new(
                fees.per_vote_certificate_fees.certificate_vote_cast,
            ),
        },
    };

    let ptr = Box::into_raw(Box::new(Settings {
        fees: linear_fee,
        discrimination,
        block0_initial_hash,
        block0_date: config::Block0Date(block0_date),
        slot_duration,
        time_era: time_era.into(),
        transaction_max_expiry_epochs,
    }));

    *settings_out = ptr;

    Result::success()
}

/// # Safety
///
///   This function also assumes that settings is a valid pointer previously
///   obtained with this library, a null check is performed, but is important that
///   the data it points to is valid
///
///   linear_fee_out must point to valid writable memory, a null check is
///   performed
pub unsafe fn settings_fees(settings: *const Settings, linear_fee_out: *mut LinearFee) -> Result {
    let settings = non_null!(settings);
    // In theory, getting a &mut from linear_fee_output and setting the fields
    // one by one should be fine, becuase is repr(C), we are not reading, and
    // the fields are just numbers (no Drop nor anything).
    // In practice, it may be UB and I don't think it's worth the hassle, so we
    // just create a new one fully initialized and use ptr::write

    let fees = settings.fees;

    let fees = LinearFee {
        constant: fees.constant,
        coefficient: fees.coefficient,
        certificate: fees.certificate,
        per_certificate_fees: PerCertificateFee {
            certificate_pool_registration: fees
                .per_certificate_fees
                .certificate_pool_registration
                .map(Into::into)
                .unwrap_or(0),
            certificate_stake_delegation: fees
                .per_certificate_fees
                .certificate_stake_delegation
                .map(Into::into)
                .unwrap_or(0),
            certificate_owner_stake_delegation: fees
                .per_certificate_fees
                .certificate_owner_stake_delegation
                .map(Into::into)
                .unwrap_or(0),
        },
        per_vote_certificate_fees: PerVoteCertificateFee {
            certificate_vote_plan: fees
                .per_vote_certificate_fees
                .certificate_vote_plan
                .map(Into::into)
                .unwrap_or(0),
            certificate_vote_cast: fees
                .per_vote_certificate_fees
                .certificate_vote_cast
                .map(Into::into)
                .unwrap_or(0),
        },
    };

    if linear_fee_out.is_null() {
        Error::invalid_input("linear_fee_out").with(NulPtr).into()
    } else {
        // we are actually not checking alignment anywhere, and I think it is
        // unlikely to get unaligned data for a struct that's essentially a u64
        // array, but just in case
        // we could just put a comment, though
        std::ptr::write_unaligned(linear_fee_out, fees);
        Result::success()
    }
}

/// # Safety
///
///   This function also assumes that settings is a valid pointer previously
///   obtained with this library, a null check is performed, but is important that
///   the data it points to is valid
///
///   discrimination_out must point to valid writable memory, a null check is
///   performed
pub unsafe fn settings_discrimination(
    settings: *const Settings,
    discrimination_out: *mut Discrimination,
) -> Result {
    let settings = non_null!(settings);

    let discrimination = match settings.discrimination {
        chain_addr::Discrimination::Production => Discrimination::Production,
        chain_addr::Discrimination::Test => Discrimination::Test,
    };

    if discrimination_out.is_null() {
        Error::invalid_input("discrimination_out")
            .with(NulPtr)
            .into()
    } else {
        // again, an enum with 2 values will probably be 1 byte, or 4 maybe...
        // maybe in this case we can assume it is aligned
        std::ptr::write_unaligned(discrimination_out, discrimination);
        Result::success()
    }
}

/// # Safety
///
///   This function assumes block0_hash points to 32 bytes of valid memory
///   This function also assumes that settings is a valid pointer previously
///   obtained with this library, a null check is performed, but is important that
///   the data it points to is valid
#[no_mangle]
pub unsafe fn settings_block0_hash(settings: *const Settings, block0_hash: *mut u8) -> Result {
    let settings = non_null!(settings);

    if block0_hash.is_null() {
        Error::invalid_input("block0_hash").with(NulPtr).into()
    } else {
        let bytes = settings.block0_initial_hash.as_bytes();

        std::ptr::copy(bytes.as_ptr(), block0_hash, bytes.len());
        Result::success()
    }
}
