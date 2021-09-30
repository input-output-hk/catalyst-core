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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::fragment::Contents;
    use crate::header::BlockVersion;
    use crate::header::ChainLength;
    use crate::header::HeaderBuilderNew;
    use crate::leadership::tests::generate_ledger_with_bft_leaders;
    use crate::leadership::tests::generate_ledger_with_bft_leaders_count;
    use crate::ledger::Pots;
    use crate::setting::Settings;
    use crate::testing::data::AddressData;
    use crate::testing::TestGen;
    use chain_crypto::Ed25519;
    fn block_date() -> BlockDate {
        BlockDate {
            epoch: 0,
            slot_id: 1,
        }
    }

    #[test]
    fn empty_bft_leaders_collection() {
        let ledger = Ledger::empty(
            Settings::new(),
            TestGen::static_parameters(),
            TestGen::time_era(),
            Pots::zero(),
        );
        assert!(LeadershipData::new(&ledger).is_none());
    }

    #[test]
    fn getters() {
        let leaders_size = 5;
        let (leaders, ledger) = generate_ledger_with_bft_leaders_count(leaders_size);
        let leadership_data =
            LeadershipData::new(&ledger).expect("leaders ids collection is empty");
        assert_eq!(leadership_data.number_of_leaders(), leaders_size);
        assert_eq!(&leaders, leadership_data.leaders());
    }

    #[test]
    fn round_robin_returns_correct_index() {
        let leaders_size = 5;
        let (leaders, ledger) = generate_ledger_with_bft_leaders_count(leaders_size);
        let leadership_data =
            LeadershipData::new(&ledger).expect("leaders ids collection is empty");

        for i in 0..leaders_size * 2 {
            assert_eq!(
                leadership_data.get_leader_at(BlockDate {
                    epoch: 0,
                    slot_id: i as u32
                }),
                leaders[i % leaders_size]
            );
        }
    }

    #[test]
    fn verify_incompatible_leadership_mode() {
        let header = TestGen::genesis_praos_header();
        let leader_key = AddressData::generate_key_pair::<Ed25519>()
            .private_key()
            .clone();
        let (_, ledger) = generate_ledger_with_bft_leaders(vec![leader_key.to_public()]);
        let leadership_data =
            LeadershipData::new(&ledger).expect("leaders ids collection is empty");

        assert!(leadership_data.verify(&header).failure());
    }

    #[test]
    fn verify_incorrect_leader() {
        let wrong_leader = AddressData::generate_key_pair::<Ed25519>()
            .private_key()
            .clone();
        let leader_key = AddressData::generate_key_pair::<Ed25519>()
            .private_key()
            .clone();
        let (_, ledger) = generate_ledger_with_bft_leaders(vec![leader_key.to_public()]);
        let leadership_data =
            LeadershipData::new(&ledger).expect("leaders ids collection is empty");

        let header = HeaderBuilderNew::new(BlockVersion::Ed25519Signed, &Contents::empty())
            .set_parent(&TestGen::hash(), ChainLength(1))
            .set_date(block_date())
            .into_bft_builder()
            .unwrap()
            .sign_using_unsafe(&leader_key, wrong_leader.to_public())
            .generalize();

        assert!(leadership_data.verify(&header).failure());
    }

    #[test]
    fn verify_incorrect_signature() {
        let wrong_leader_key = AddressData::generate_key_pair::<Ed25519>()
            .private_key()
            .clone();
        let leader_key = AddressData::generate_key_pair::<Ed25519>()
            .private_key()
            .clone();
        let (_, ledger) = generate_ledger_with_bft_leaders(vec![leader_key.to_public()]);
        let leadership_data =
            LeadershipData::new(&ledger).expect("leaders ids collection is empty");

        let header = HeaderBuilderNew::new(BlockVersion::Ed25519Signed, &Contents::empty())
            .set_parent(&TestGen::hash(), ChainLength(1))
            .set_date(block_date())
            .into_bft_builder()
            .unwrap()
            .sign_using_unsafe(&wrong_leader_key, leader_key.to_public())
            .generalize();

        assert!(leadership_data.verify(&header).failure());
    }

    #[test]
    fn verify_correct_verification() {
        let leader_key = AddressData::generate_key_pair::<Ed25519>()
            .private_key()
            .clone();
        let (_, ledger) = generate_ledger_with_bft_leaders(vec![leader_key.to_public()]);
        let leadership_data =
            LeadershipData::new(&ledger).expect("leaders ids collection is empty");

        let header = HeaderBuilderNew::new(BlockVersion::Ed25519Signed, &Contents::empty())
            .set_parent(&TestGen::hash(), ChainLength(1))
            .set_date(block_date())
            .into_bft_builder()
            .unwrap()
            .sign_using_unsafe(&leader_key, leader_key.to_public())
            .generalize();

        assert!(leadership_data.verify(&header).success());
    }
}
