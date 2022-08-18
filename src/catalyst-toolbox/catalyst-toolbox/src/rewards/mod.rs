pub mod community_advisors;
pub mod dreps;
pub mod veterans;
pub mod voters;

use rust_decimal::Decimal;
pub type Funds = Decimal;
// Lets match to the same type as the funds, but naming it funds would be confusing
pub type Rewards = Decimal;
pub type VoteCount = HashMap<Identifier, HashSet<Hash>>;

use jormungandr_lib::crypto::{account::Identifier, hash::Hash};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;

#[derive(Debug, Error)]
pub enum Error {
    #[error("hash is not a valid blake2b256 hash")]
    InvalidHash(Vec<u8>),
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
                <[u8; 32]>::try_from(p.proposal.chain_proposal_id)
                    .map_err(Error::InvalidHash)
                    .map(|hash| (p.proposal.challenge_id, Hash::from(hash)))
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
        if votes.len() < self.total {
            return false;
        }

        for (challenge, threshold) in &self.per_challenge {
            let votes_in_challengs = self
                .proposals_per_challenge
                .get(challenge)
                .map(|props| votes.intersection(props).count())
                .unwrap_or_default();
            if votes_in_challengs < *threshold {
                return false;
            }
        }

        true
    }
}
