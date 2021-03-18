use super::NulPtr;
use crate::{Error, Result};
use chain_impl_mockchain::{fee, header::HeaderId};
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

pub unsafe fn settings_new(
    fees: LinearFee,
    discrimination: Discrimination,
    block_0_hash: *const u8,
    settings_out: *mut *mut Settings,
) -> Result {
    let settings_out = non_null_mut!(settings_out);

    let discrimination = match discrimination {
        Discrimination::Production => chain_addr::Discrimination::Production,
        Discrimination::Test => chain_addr::Discrimination::Test,
    };

    let block0_initial_hash =
        HeaderId::from_bytes(non_null_array!(block_0_hash, 32).try_into().unwrap());

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
    }));

    *settings_out = ptr;

    Result::success()
}

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
