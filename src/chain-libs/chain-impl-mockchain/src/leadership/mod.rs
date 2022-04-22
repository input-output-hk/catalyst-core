use crate::{
    block::{BlockDate, BlockVersion, Header},
    certificate::PoolId,
    chaintypes::ConsensusType,
    date::Epoch,
    key::BftLeaderId,
    ledger::{Ledger, LedgerParameters},
    stake::StakeDistribution,
};
use chain_crypto::{Ed25519, RistrettoGroup2HashDh, SecretKey, SumEd25519_12};
use chain_time::era::TimeEra;

pub mod bft;
pub mod genesis;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Failure,
    NoLeaderForThisSlot,
    IncompatibleBlockVersion,
    IncompatibleLeadershipMode,
    InvalidLeader,
    InvalidLeaderSignature,
    InvalidLeaderProof,
    InvalidBlockMessage,
    InvalidStateUpdate,
    VrfNonceIsEmptyButNotSupposedTo,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

/// Verification type for when validating a block
#[derive(Debug)]
pub enum Verification {
    Success,
    Failure(Error),
}

macro_rules! try_check {
    ($x:expr) => {
        if $x.failure() {
            return $x;
        }
    };
}

pub struct BftLeader {
    pub sig_key: SecretKey<Ed25519>,
}

impl From<SecretKey<Ed25519>> for Leader {
    fn from(secret_key: SecretKey<Ed25519>) -> Self {
        Leader {
            bft_leader: Some(BftLeader {
                sig_key: secret_key,
            }),
            genesis_leader: None,
        }
    }
}

pub struct GenesisLeader {
    pub node_id: PoolId,
    pub sig_key: SecretKey<SumEd25519_12>,
    pub vrf_key: SecretKey<RistrettoGroup2HashDh>,
}

pub struct Leader {
    pub bft_leader: Option<BftLeader>,
    pub genesis_leader: Option<GenesisLeader>,
}

#[allow(clippy::large_enum_variant)]
pub enum LeaderOutput {
    None,
    Bft(BftLeaderId),
    GenesisPraos(PoolId, genesis::Witness),
}

pub enum LeadershipConsensus {
    Bft(bft::LeadershipData),
    GenesisPraos(genesis::LeadershipData),
}

/// Leadership represent a given epoch and their associated leader or metadata.
pub struct Leadership {
    // Specific epoch where the leadership apply
    epoch: Epoch,
    // Give the closest parameters associated with date keeping given a leadership
    era: TimeEra,
    // Consensus specific metadata required for verifying/evaluating leaders
    inner: LeadershipConsensus,
    // Ledger evaluation parameters fixed for a given epoch
    ledger_parameters: LedgerParameters,
}

impl LeadershipConsensus {
    #[inline]
    fn verify_version(&self, block_version: BlockVersion) -> Verification {
        match self {
            LeadershipConsensus::Bft(_) if block_version == BlockVersion::Ed25519Signed => {
                Verification::Success
            }
            LeadershipConsensus::GenesisPraos(_) if block_version == BlockVersion::KesVrfproof => {
                Verification::Success
            }
            _ => Verification::Failure(Error::new(ErrorKind::IncompatibleBlockVersion)),
        }
    }

    #[inline]
    fn verify_leader(&self, block_header: &Header) -> Verification {
        match self {
            LeadershipConsensus::Bft(bft) => bft.verify(block_header),
            LeadershipConsensus::GenesisPraos(genesis_praos) => genesis_praos.verify(block_header),
        }
    }

    #[inline]
    fn is_leader(&self, leader: &Leader, date: BlockDate) -> LeaderOutput {
        match self {
            LeadershipConsensus::Bft(bft) => match leader.bft_leader {
                Some(ref bft_leader) => {
                    let bft_leader_id = bft.get_leader_at(date);
                    if bft_leader_id == bft_leader.sig_key.to_public().into() {
                        LeaderOutput::Bft(bft_leader_id)
                    } else {
                        LeaderOutput::None
                    }
                }
                None => LeaderOutput::None,
            },
            LeadershipConsensus::GenesisPraos(genesis_praos) => match leader.genesis_leader {
                None => LeaderOutput::None,
                Some(ref gen_leader) => {
                    match genesis_praos.leader(&gen_leader.node_id, &gen_leader.vrf_key, date) {
                        Ok(Some(witness)) => {
                            LeaderOutput::GenesisPraos(gen_leader.node_id.clone(), witness)
                        }
                        _ => LeaderOutput::None,
                    }
                }
            },
        }
    }
}

impl Leadership {
    pub fn new(epoch: Epoch, ledger: &Ledger) -> Self {
        let inner = match ledger.settings.consensus_version {
            ConsensusType::Bft => {
                LeadershipConsensus::Bft(bft::LeadershipData::new(ledger).unwrap())
            }
            ConsensusType::GenesisPraos => {
                LeadershipConsensus::GenesisPraos(genesis::LeadershipData::new(epoch, ledger))
            }
        };
        Leadership {
            epoch,
            era: ledger.era.clone(),
            inner,
            ledger_parameters: ledger.get_ledger_parameters(),
        }
    }

    /// get the epoch associated to the `Leadership`
    #[inline]
    pub fn epoch(&self) -> Epoch {
        self.epoch
    }

    pub fn stake_distribution(&self) -> Option<&StakeDistribution> {
        match &self.inner {
            LeadershipConsensus::Bft(_) => None,
            LeadershipConsensus::GenesisPraos(inner) => Some(inner.distribution()),
        }
    }

    /// Create a Block date given a leadership and a relative epoch slot
    ///
    /// # Panics
    ///
    /// If the slot index is not valid given the leadership, out of bound date
    pub fn date_at_slot(&self, slot_id: u32) -> BlockDate {
        assert!(slot_id < self.era.slots_per_epoch());
        BlockDate {
            epoch: self.epoch(),
            slot_id,
        }
    }

    /// get the TimeEra associated to the `Leadership`
    #[inline]
    pub fn era(&self) -> &TimeEra {
        &self.era
    }

    /// get the consensus associated with the `Leadership`
    pub fn consensus(&self) -> &LeadershipConsensus {
        &self.inner
    }

    /// access the ledger parameter for the current leadership
    #[inline]
    pub fn ledger_parameters(&self) -> &LedgerParameters {
        &self.ledger_parameters
    }

    /// Verify whether this header has been produced by a leader that fits with the leadership
    ///
    pub fn verify(&self, block_header: &Header) -> Verification {
        try_check!(self.inner.verify_version(block_header.block_version()));

        try_check!(self.inner.verify_leader(block_header));
        Verification::Success
    }

    /// Test that the given leader object is able to create a valid block for the leadership
    /// at a given date.
    pub fn is_leader_for_date(&self, leader: &Leader, date: BlockDate) -> LeaderOutput {
        self.inner.is_leader(leader, date)
    }
}

impl Verification {
    #[inline]
    pub fn into_error(self) -> Result<(), Error> {
        match self {
            Verification::Success => Ok(()),
            Verification::Failure(err) => Err(err),
        }
    }
    #[inline]
    pub fn success(&self) -> bool {
        matches!(self, Verification::Success)
    }
    #[inline]
    pub fn failure(&self) -> bool {
        !self.success()
    }
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Error { kind, cause: None }
    }

    pub fn new_<E>(kind: ErrorKind, cause: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error {
            kind,
            cause: Some(Box::new(cause)),
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrorKind::Failure => write!(f, "The current state of the leader selection is invalid"),
            ErrorKind::NoLeaderForThisSlot => write!(f, "No leader available for this block date"),
            ErrorKind::IncompatibleBlockVersion => {
                write!(f, "The block Version is incompatible with LeaderSelection.")
            }
            ErrorKind::IncompatibleLeadershipMode => {
                write!(f, "Incompatible leadership mode (the proof is invalid)")
            }
            ErrorKind::InvalidLeader => write!(f, "Block has unexpected block leader"),
            ErrorKind::InvalidLeaderSignature => write!(f, "Block signature is invalid"),
            ErrorKind::InvalidLeaderProof => write!(f, "Block proof is invalid"),
            ErrorKind::InvalidBlockMessage => write!(f, "Invalid block message"),
            ErrorKind::InvalidStateUpdate => write!(f, "Invalid State Update"),
            ErrorKind::VrfNonceIsEmptyButNotSupposedTo => write!(f, "Vrf Nonce is empty"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(cause) = &self.cause {
            write!(f, "{}: {}", self.kind, cause)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_ref()
            .map(|cause| -> &(dyn std::error::Error + 'static) { cause.as_ref() })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::fragment::Contents;
    use crate::header::ChainLength;
    use crate::header::HeaderBuilderNew;
    use crate::testing::data::AddressData;
    use crate::testing::ConfigBuilder;
    use crate::testing::LedgerBuilder;
    use crate::testing::TestGen;
    use chain_crypto::PublicKey;

    pub fn generate_ledger_with_bft_leaders_count(count: usize) -> (Vec<BftLeaderId>, Ledger) {
        let leaders: Vec<PublicKey<Ed25519>> = TestGen::leaders_pairs()
            .take(count)
            .map(|x| x.key().to_public())
            .collect();
        generate_ledger_with_bft_leaders(leaders)
    }

    pub fn generate_ledger_with_bft_leaders(
        leaders: Vec<PublicKey<Ed25519>>,
    ) -> (Vec<BftLeaderId>, Ledger) {
        let leaders: Vec<BftLeaderId> = leaders.into_iter().map(BftLeaderId).collect();
        let config = ConfigBuilder::new().with_leaders(&leaders);
        let test_ledger = LedgerBuilder::from_config(config)
            .build()
            .expect("cannot build ledger");
        (leaders, test_ledger.ledger)
    }

    pub fn generate_header_for_leader(leader_key: SecretKey<Ed25519>, slot_id: u32) -> Header {
        HeaderBuilderNew::new(BlockVersion::Ed25519Signed, &Contents::empty())
            .set_parent(&TestGen::hash(), ChainLength(slot_id))
            .set_date(BlockDate { epoch: 0, slot_id })
            .into_bft_builder()
            .unwrap()
            .sign_using(&leader_key)
            .generalize()
    }

    #[test]
    fn consensus_verify_version_for_bft() {
        let ledger = TestGen::ledger();

        let data = bft::LeadershipData::new(&ledger).expect("Couldn't build leadership data");

        let bft_leadership_consensus = LeadershipConsensus::Bft(data);

        assert!(bft_leadership_consensus
            .verify_version(BlockVersion::Ed25519Signed)
            .success());
        assert!(bft_leadership_consensus
            .verify_version(BlockVersion::KesVrfproof)
            .failure());
        assert!(bft_leadership_consensus
            .verify_version(BlockVersion::Genesis)
            .failure());
    }

    #[test]
    fn consensus_leader_for_bft() {
        let leader_key = AddressData::generate_key_pair::<Ed25519>();
        let (_, ledger) = generate_ledger_with_bft_leaders(vec![leader_key.public_key().clone()]);
        let leadership_data =
            bft::LeadershipData::new(&ledger).expect("leaders ids collection is empty");
        let bft_leadership_consensus = LeadershipConsensus::Bft(leadership_data);
        let header = generate_header_for_leader(leader_key.private_key().clone(), 0);
        assert!(bft_leadership_consensus.verify_leader(&header).success());
    }

    #[test]
    fn is_leader_for_bft_correct() {
        let leaders_count = 5;
        let leaders_keys: Vec<SecretKey<Ed25519>> =
            TestGen::secret_keys().take(leaders_count).collect();
        let (leaders, ledger) =
            generate_ledger_with_bft_leaders(leaders_keys.iter().map(|x| x.to_public()).collect());
        let leadership_data =
            bft::LeadershipData::new(&ledger).expect("leaders ids collection is empty");
        let bft_leadership_consensus = LeadershipConsensus::Bft(leadership_data);

        for leader_index in 0..leaders_count {
            let leader_output = bft_leadership_consensus.is_leader(
                &leaders_keys[leader_index].clone().into(),
                BlockDate {
                    epoch: 0,
                    slot_id: leader_index as u32,
                },
            );
            if let LeaderOutput::Bft(actual_leader) = leader_output {
                assert_eq!(leaders[leader_index].clone(), actual_leader);
            } else {
                panic!("wrong leader at index: {}", leader_index);
            }
        }
    }

    #[test]
    fn is_leader_for_bft_negative() {
        let leaders_count = 5;
        let leaders_keys: Vec<SecretKey<Ed25519>> =
            TestGen::secret_keys().take(leaders_count).collect();
        let (_, ledger) =
            generate_ledger_with_bft_leaders(leaders_keys.iter().map(|x| x.to_public()).collect());

        let leadership_data =
            bft::LeadershipData::new(&ledger).expect("leaders ids collection is empty");
        let bft_leadership_consensus = LeadershipConsensus::Bft(leadership_data);

        for leader in leaders_keys.iter().take(leaders_count - 1).cloned() {
            let _leader_output = bft_leadership_consensus.is_leader(
                &leader.into(),
                BlockDate {
                    epoch: 0,
                    slot_id: leaders_count as u32,
                },
            );
            assert!(matches!(LeaderOutput::None, _leader_output));
        }
    }

    #[test]
    fn is_leader_empty() {
        let ledger = TestGen::ledger();

        let leadership_data =
            bft::LeadershipData::new(&ledger).expect("leaders ids collection is empty");
        let bft_leadership_consensus = LeadershipConsensus::Bft(leadership_data);

        let leader = Leader {
            bft_leader: None,
            genesis_leader: None,
        };
        let _leader_output = bft_leadership_consensus.is_leader(
            &leader,
            BlockDate {
                epoch: 0,
                slot_id: 0,
            },
        );
        assert!(matches!(LeaderOutput::None, _leader_output));
    }

    #[test]
    fn consensus_verify_version_for_praos() {
        let ledger = TestGen::ledger();

        let data = genesis::LeadershipData::new(0, &ledger);

        let gen_leadership_consensus = LeadershipConsensus::GenesisPraos(data);

        assert!(gen_leadership_consensus
            .verify_version(BlockVersion::Ed25519Signed)
            .failure());
        assert!(gen_leadership_consensus
            .verify_version(BlockVersion::KesVrfproof)
            .success());
        assert!(gen_leadership_consensus
            .verify_version(BlockVersion::Genesis)
            .failure());
    }

    #[test]
    #[should_panic]
    fn ledership_getters_bft_slot_no_negative() {
        let test_ledger = LedgerBuilder::from_config(
            ConfigBuilder::new()
                .with_consensus_version(ConsensusType::Bft)
                .with_slots_per_epoch(60),
        )
        .build()
        .unwrap();

        let leadership = Leadership::new(0, &test_ledger.ledger);
        leadership.date_at_slot(61);
    }

    #[test]
    fn ledership_getters_bft() {
        let epoch = 0;
        let slot_id = 1;

        let test_ledger = LedgerBuilder::from_config(
            ConfigBuilder::new()
                .with_consensus_version(ConsensusType::Bft)
                .with_slots_per_epoch(60),
        )
        .build()
        .unwrap();

        let leadership = Leadership::new(epoch, &test_ledger.ledger);

        assert_eq!(leadership.epoch(), epoch);
        assert_eq!(
            leadership.date_at_slot(slot_id),
            BlockDate { epoch: 0, slot_id }
        );
        assert_eq!(leadership.stake_distribution(), None);
    }

    #[test]
    fn ledership_getters_praos() {
        let epoch = 0;
        let slot_id = 1;

        let test_ledger = LedgerBuilder::from_config(
            ConfigBuilder::new()
                .with_consensus_version(ConsensusType::GenesisPraos)
                .with_slots_per_epoch(60),
        )
        .build()
        .unwrap();

        let leadership = Leadership::new(epoch, &test_ledger.ledger);

        assert_eq!(leadership.epoch(), epoch);
        assert_eq!(
            leadership.date_at_slot(slot_id),
            BlockDate { epoch: 0, slot_id }
        );
        assert_eq!(leadership.stake_distribution(),Some(&test_ledger.ledger.get_stake_distribution()));
    }
    
    #[test]
    fn leadership_is_leader_for_date() {
        let leaders_count = 5;
        let leaders_keys: Vec<SecretKey<Ed25519>> =
            TestGen::secret_keys().take(leaders_count).collect();
        let leaders: Vec<BftLeaderId> = leaders_keys
            .iter()
            .map(|x| BftLeaderId(x.to_public()))
            .collect();
        let config = ConfigBuilder::new()
            .with_leaders(&leaders)
            .with_consensus_version(ConsensusType::Bft)
            .with_slots_per_epoch(60);
        let test_ledger = LedgerBuilder::from_config(config)
            .build()
            .expect("cannot build ledger");

        let leadership = Leadership::new(0, &test_ledger.ledger);

        for leader_index in 0..leaders_count {
            let leader_output = leadership.is_leader_for_date(
                &leaders_keys[leader_index].clone().into(),
                BlockDate {
                    epoch: 0,
                    slot_id: leader_index as u32,
                },
            );
            if let LeaderOutput::Bft(actual_leader) = leader_output {
                assert_eq!(leaders[leader_index].clone(), actual_leader);
            } else {
                panic!("wrong leader at index: {}", leader_index);
            }
        }
    }

    #[test]
    fn leadership_verify() {
        let leaders_count = 5usize;
        let leaders_keys: Vec<SecretKey<Ed25519>> =
            TestGen::secret_keys().take(leaders_count).collect();
        let leaders: Vec<BftLeaderId> = leaders_keys.iter().map(|x| x.to_public().into()).collect();
        let config = ConfigBuilder::new()
            .with_leaders(&leaders)
            .with_consensus_version(ConsensusType::Bft)
            .with_slots_per_epoch(60);
        let test_ledger = LedgerBuilder::from_config(config)
            .build()
            .expect("cannot build ledger");

        let leadership = Leadership::new(0, &test_ledger.ledger);

        for (i, key) in leaders_keys.iter().cloned().enumerate() {
            let header = generate_header_for_leader(key, i as u32);
            assert!(leadership.verify(&header).success());
        }
    }
}
