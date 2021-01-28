use super::ProposalPtr;
use crate::{
    vote::{EncryptingVoteKey, PayloadTypeConfig},
    Error, Proposal, Result as AbiResult,
};
use chain_impl_mockchain::{certificate::VotePlanId, vote::Options as VoteOptions};
use std::convert::TryFrom;
use std::ffi::CStr;
use thiserror::Error;
pub use wallet::Settings;

const ENCRYPTION_VOTE_KEY_HRP: &str = "p256k1_votepk";

// using generics in this module is questionable, but it's used just for code
// re-use, the idea is to have two functions, and then it's exposed that way in
// wallet-c/wallet-jni (with manual name mangling).
// for the C interface, a tagged union could be used as input too, but I think
// using the same approach for all the interfaces it's better.
// something else that could work is a new opaque type.
pub struct ProposalPublic;
pub struct ProposalPrivate<'a>(pub &'a CStr);

#[derive(Error, Debug)]
#[error("invalid binary format")]
pub struct InvalidEncryptionKey;
#[derive(Error, Debug)]
#[error("bech32 string is not valid")]
pub struct InvalidBech32;

pub trait ToPayload {
    fn to_payload(self) -> Result<PayloadTypeConfig, Error>;
}

impl ToPayload for ProposalPublic {
    fn to_payload(self) -> Result<PayloadTypeConfig, Error> {
        Ok(PayloadTypeConfig::Public)
    }
}

impl<'a> ToPayload for ProposalPrivate<'a> {
    fn to_payload(self) -> Result<PayloadTypeConfig, Error> {
        use bech32::FromBase32;
        self.0
            .to_str()
            .map_err(|_| Error::invalid_input("encrypting_vote_key"))
            .and_then(|s| {
                bech32::decode(s)
                    .map_err(|_| Error::invalid_input("encrypting_vote_key").with(InvalidBech32))
            })
            .and_then(|(hrp, raw_key)| {
                if hrp != ENCRYPTION_VOTE_KEY_HRP {
                    return Err(Error::invalid_bech32_hrp(ENCRYPTION_VOTE_KEY_HRP, hrp));
                }

                let bytes = Vec::<u8>::from_base32(&raw_key).unwrap();

                EncryptingVoteKey::from_bytes(&bytes).ok_or_else(|| {
                    Error::invalid_input("encrypting_vote_key").with(InvalidEncryptionKey)
                })
            })
            .map(PayloadTypeConfig::Private)
            .map_err(|_| Error::invalid_input("encrypting_vote_key").with(InvalidEncryptionKey))
    }
}

/// build the proposal object
///
/// # Errors
///
/// This function may fail if:
///
/// * a null pointer was provided as an argument.
/// * `num_choices` is out of the allowed range.
///
/// # Safety
///
/// This function dereference raw pointers. Even though the function checks if
/// the pointers are null. Mind not to put random values in or you may see
/// unexpected behaviors.
pub unsafe fn proposal_new<P: ToPayload>(
    vote_plan_id: *const u8,
    index: u8,
    num_choices: u8,
    payload_type: P,
    proposal_out: *mut ProposalPtr,
) -> AbiResult {
    let options = match VoteOptions::new_length(num_choices) {
        Ok(options) => options,
        Err(err) => return Error::invalid_input("num_choices").with(err).into(),
    };

    let vote_plan_id = non_null_array!(vote_plan_id, crate::vote::VOTE_PLAN_ID_LENGTH);
    let vote_plan_id = match VotePlanId::try_from(vote_plan_id) {
        Ok(id) => id,
        Err(err) => return Error::invalid_input("vote_plan_id").with(err).into(),
    };

    let payload_type = match payload_type.to_payload() {
        Ok(payload_type) => payload_type,
        Err(err) => return err.into(),
    };

    let proposal = Proposal::new(vote_plan_id, index, options, payload_type);
    *non_null_mut!(proposal_out) = Box::into_raw(Box::new(proposal));

    AbiResult::success()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bech32::ToBase32;

    #[test]
    fn cast_private_vote() {
        use chain_vote::gargamel;
        use rand::SeedableRng;
        let vote_plan_id = [0u8; crate::vote::VOTE_PLAN_ID_LENGTH];
        let mut rng = rand_chacha::ChaCha20Rng::from_seed([1u8; 32]);
        let sk = gargamel::SecretKey::generate(&mut rng);
        let pk = gargamel::Keypair::from_secretkey(sk).public_key;

        let encrypting_vote_key =
            bech32::encode(ENCRYPTION_VOTE_KEY_HRP, pk.to_bytes().to_base32()).unwrap();
        let encrypting_vote_key = std::ffi::CString::new(encrypting_vote_key).unwrap();

        let mut proposal: ProposalPtr = std::ptr::null_mut();
        unsafe {
            let result = proposal_new(
                vote_plan_id.as_ptr(),
                0,
                2,
                ProposalPrivate(&encrypting_vote_key),
                (&mut proposal) as *mut ProposalPtr,
            );
            assert!(result.is_ok());
        }
    }
}
