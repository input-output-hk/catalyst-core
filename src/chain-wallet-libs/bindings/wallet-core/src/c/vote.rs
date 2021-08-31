use super::ProposalPtr;
use crate::{vote::PayloadTypeConfig, Error, Proposal, Result as AbiResult};
use chain_crypto::bech32::Bech32;
use chain_impl_mockchain::{certificate::VotePlanId, vote::Options as VoteOptions};
use chain_vote::ElectionPublicKey;
use std::convert::{TryFrom, TryInto};
use std::ffi::CStr;
pub use wallet::Settings;

// using generics in this module is questionable, but it's used just for code
// re-use, the idea is to have two functions, and then it's exposed that way in
// wallet-c/wallet-jni (with manual name mangling).
// for the C interface, a tagged union could be used as input too, but I think
// using the same approach for all the interfaces it's better.
// something else that could work is a new opaque type.
pub struct ProposalPublic;
pub struct ProposalPrivate<'a>(pub &'a CStr);

impl TryInto<PayloadTypeConfig> for ProposalPublic {
    type Error = Error;

    fn try_into(self) -> Result<PayloadTypeConfig, Error> {
        Ok(PayloadTypeConfig::Public)
    }
}

impl<'a> TryInto<PayloadTypeConfig> for ProposalPrivate<'a> {
    type Error = Error;

    fn try_into(self) -> Result<PayloadTypeConfig, Error> {
        const INPUT_NAME: &str = "election_public_key";

        self.0
            .to_str()
            .map_err(|_| Error::invalid_input(INPUT_NAME))
            .and_then(|s| {
                ElectionPublicKey::try_from_bech32_str(s)
                    .map_err(|_| Error::invalid_vote_encryption_key())
            })
            .map(PayloadTypeConfig::Private)
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
pub unsafe fn proposal_new<P>(
    vote_plan_id: *const u8,
    index: u8,
    num_choices: u8,
    payload_type: P,
    proposal_out: *mut ProposalPtr,
) -> AbiResult
where
    P: TryInto<PayloadTypeConfig>,
    P::Error: Into<AbiResult>,
{
    let options = match VoteOptions::new_length(num_choices) {
        Ok(options) => options,
        Err(err) => return Error::invalid_input("num_choices").with(err).into(),
    };

    let vote_plan_id = non_null_array!(vote_plan_id, crate::vote::VOTE_PLAN_ID_LENGTH);
    let vote_plan_id = match VotePlanId::try_from(vote_plan_id) {
        Ok(id) => id,
        Err(err) => return Error::invalid_input("vote_plan_id").with(err).into(),
    };

    let payload_type = match payload_type.try_into() {
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

    #[test]
    fn cast_private_vote() {
        use chain_vote::{
            committee::{MemberCommunicationKey, MemberState},
            tally::Crs,
        };
        let vote_plan_id = [0u8; crate::vote::VOTE_PLAN_ID_LENGTH];

        let shared_string =
            b"Example of a shared string. This should be VotePlan.to_id()".to_owned();
        let h = Crs::from_hash(&shared_string);

        let mut rng = rand::thread_rng();

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public()];

        let threshold = 1;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);

        let pk = ElectionPublicKey::from_participants(&[m1.public_key()]);

        let election_public_key = pk.to_bech32_str();
        let election_public_key = std::ffi::CString::new(election_public_key).unwrap();

        let mut proposal: ProposalPtr = std::ptr::null_mut();
        unsafe {
            let result = proposal_new(
                vote_plan_id.as_ptr(),
                0,
                2,
                ProposalPrivate(&election_public_key),
                (&mut proposal) as *mut ProposalPtr,
            );
            assert!(result.is_ok());
        }
    }
}
