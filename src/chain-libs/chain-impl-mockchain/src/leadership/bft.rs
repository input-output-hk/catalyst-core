use crate::block::{BftProof, BlockDate, Header, Proof};
use crate::{
    key::BftLeaderId,
    leadership::{Error, ErrorKind, Verification},
    ledger::Ledger,
};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BftRoundRobinIndex(u64);

/// The BFT Leader selection is based on a round robin of the expected leaders
#[derive(Debug)]
pub struct LeadershipData {
    pub(crate) leaders: Arc<[BftLeaderId]>,
}

impl LeadershipData {
    /// Create a new BFT leadership
    pub fn new(ledger: &Ledger) -> Option<Self> {
        if ledger.settings.bft_leaders.len() == 0 {
            return None;
        }

        Some(LeadershipData {
            leaders: Arc::clone(&ledger.settings.bft_leaders),
        })
    }

    #[inline]
    pub fn number_of_leaders(&self) -> usize {
        self.leaders.len()
    }

    pub fn leaders(&self) -> &[BftLeaderId] {
        self.leaders.as_ref()
    }

    #[inline]
    fn offset(&self, block_number: u64) -> BftRoundRobinIndex {
        let max = self.number_of_leaders() as u64;
        BftRoundRobinIndex((block_number % max) as u64)
    }

    pub(crate) fn verify(&self, block_header: &Header) -> Verification {
        match &block_header.proof() {
            Proof::Bft(bft_proof) => {
                let BftProof {
                    leader_id,
                    signature,
                } = bft_proof;

                let leader = self.get_leader_at(block_header.block_date());

                if leader_id != &leader {
                    Verification::Failure(Error::new(ErrorKind::InvalidLeader))
                } else {
                    // verify block signature
                    match signature
                        .0
                        .verify_slice(leader.as_public_key(), block_header.as_auth_slice())
                    {
                        chain_crypto::Verification::Failed => {
                            Verification::Failure(Error::new(ErrorKind::InvalidLeaderSignature))
                        }
                        chain_crypto::Verification::Success => Verification::Success,
                    }
                }
            }
            _ => Verification::Failure(Error::new(ErrorKind::IncompatibleLeadershipMode)),
        }
    }

    #[inline]
    pub(crate) fn get_leader_at(&self, date: BlockDate) -> BftLeaderId {
        let BftRoundRobinIndex(ofs) = self.offset(date.slot_id as u64);
        self.leaders[ofs as usize].clone()
    }
}
