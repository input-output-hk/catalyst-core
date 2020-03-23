use crate::block::{BlockDate, Header, Proof};
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
    pub(crate) leaders: Arc<Vec<BftLeaderId>>,
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

    #[inline]
    fn offset(&self, block_number: u64) -> BftRoundRobinIndex {
        let max = self.number_of_leaders() as u64;
        BftRoundRobinIndex((block_number % max) as u64)
    }

    pub(crate) fn verify(&self, block_header: &Header) -> Verification {
        match &block_header.proof() {
            Proof::Bft(bft_proof) => match self.get_leader_at(block_header.block_date()) {
                Ok(leader_at) => {
                    if bft_proof.leader_id != leader_at {
                        Verification::Failure(Error::new(ErrorKind::InvalidLeader))
                    } else {
                        Verification::Success
                    }
                }
                Err(error) => Verification::Failure(error),
            },
            _ => Verification::Failure(Error::new(ErrorKind::InvalidLeaderSignature)),
        }
    }

    #[inline]
    pub(crate) fn get_leader_at(&self, date: BlockDate) -> Result<BftLeaderId, Error> {
        let BftRoundRobinIndex(ofs) = self.offset(date.slot_id as u64);
        Ok(self.leaders[ofs as usize].clone())
    }
}
