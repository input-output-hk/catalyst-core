pub mod community_advisors;
pub mod dreps;
pub mod proposers;
pub mod veterans;
pub mod voters;

use rust_decimal::Decimal;
pub type Funds = Decimal;
// Lets match to the same type as the funds, but naming it funds would be confusing
pub type Rewards = Decimal;
pub type VoteCount = HashMap<Identifier, HashSet<Hash>>;

use chain_impl_mockchain::certificate::ExternalProposalId;
use jormungandr_lib::crypto::{account::Identifier, hash::Hash};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use thiserror::Error;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

#[derive(Debug, Error)]
pub enum Error {
    #[error("hash is not a valid blake2b256 hash")]
    InvalidHash(Vec<u8>),
}

/// Convert a slice of bytes that represent a valid `ExternalProposalId`, and returns an array of
/// the decoded 32-byte array.
pub(crate) fn chain_proposal_id_bytes(v: &[u8]) -> Result<[u8; 32], Error> {
    // the chain_proposal_id comes as hex-encoded string digest of a blake2b256 key
    // the first step is to decode the &str
    let chain_proposal_str =
        std::str::from_utf8(v).map_err(|_| Error::InvalidHash(v.to_owned()))?;
    // second step is to convert &str into a digest so that it can be converted into
    // [u8;32]
    let chain_proposal_id = ExternalProposalId::from_str(chain_proposal_str)
        .map_err(|_| Error::InvalidHash(v.to_owned()))?;
    let bytes: [u8; 32] = chain_proposal_id.into();
    Ok(bytes)
}

pub struct Threshold {
    total: usize,
    per_challenge: HashMap<i32, usize>,
    proposals_per_challenge: HashMap<i32, HashSet<Hash>>,
}

impl Threshold {
    pub fn new(
        total_threshold: usize,
        per_challenge: HashMap<i32, usize>,
        proposals: Vec<FullProposalInfo>,
    ) -> Result<Self, Error> {
        let proposals = proposals
            .into_iter()
            .map(|p| {
                let bytes = chain_proposal_id_bytes(&p.proposal.chain_proposal_id)?;
                Ok((p.proposal.challenge_id, Hash::from(bytes)))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Self {
            total: total_threshold,
            per_challenge,
            proposals_per_challenge: proposals.into_iter().fold(
                HashMap::new(),
                |mut acc, (challenge_id, hash)| {
                    acc.entry(challenge_id).or_default().insert(hash);
                    acc
                },
            ),
        })
    }

    fn filter(&self, votes: &HashSet<Hash>) -> bool {

        println!("Threshold::filter: votes.len() = {:?}", votes.len());

        if votes.len() < self.total {
            println!("Threshold::filter: votes.len() < self.total");
            return false;
        }

        for (challenge, threshold) in &self.per_challenge {
            let votes_in_challenges = self
                .proposals_per_challenge
                .get(challenge)
                .map(|props| votes.intersection(props).count())
                .unwrap_or_default();

            println!("Threshold::filter: challenge = {:?}", *challenge);
            println!("Threshold::filter: votes_in_challenges = {:?}", votes_in_challenges);
            println!("Threshold::filter: threshold = {:?}", *threshold);

            if votes_in_challenges < *threshold {
                return false;
            }
        }

        true
    }
}
