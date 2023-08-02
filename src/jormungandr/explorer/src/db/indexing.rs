use super::{error, persistent_sequence::PersistentSequence};
use cardano_legacy_address::Addr as OldAddress;
use chain_addr::{Address, Discrimination};
use chain_core::property::{Block as _, Fragment as _};
use chain_impl_mockchain::{
    account::Identifier,
    block::{Block, Proof},
    certificate::{
        Certificate, ExternalProposalId, PoolId, PoolRegistration, PoolRetirement, VotePlanId,
    },
    fragment::{ConfigParams, Fragment, FragmentId},
    header::{BlockDate, ChainLength, Epoch, HeaderId as HeaderHash},
    key::{BftLeaderId, Hash},
    transaction::{InputEnum, TransactionSlice, Witness},
    value::Value,
    vote::{Choice, EncryptedVote, Options, PayloadType, ProofOfCorrectVote, Weight},
};
use error::ExplorerError as Error;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    convert::TryInto,
    sync::Arc,
};

pub type Hamt<K, V> = imhamt::Hamt<DefaultHasher, K, Arc<V>>;

pub type Transactions = Hamt<FragmentId, HeaderHash>;
pub type Blocks = Hamt<HeaderHash, ExplorerBlock>;
pub type ChainLengths = Hamt<ChainLength, HeaderHash>;

pub type Addresses = Hamt<ExplorerAddress, PersistentSequence<FragmentId>>;
pub type Epochs = Hamt<Epoch, EpochData>;

pub type StakePoolBlocks = Hamt<PoolId, PersistentSequence<HeaderHash>>;
pub type StakePool = Hamt<PoolId, StakePoolData>;

pub type VotePlans = Hamt<VotePlanId, ExplorerVotePlan>;

#[derive(Clone, Debug)]
pub struct StakePoolData {
    pub registration: PoolRegistration,
    pub retirement: Option<PoolRetirement>,
    // TODO: Track updates here too?
}

/// Block with unified inputs the metadata needed in the queries
#[derive(Clone, Debug)]
pub struct ExplorerBlock {
    /// The HashMap allows for easy search when querying transactions by id
    pub transactions: HashMap<FragmentId, ExplorerTransaction>,
    pub id: HeaderHash,
    pub date: BlockDate,
    pub chain_length: ChainLength,
    pub parent_hash: HeaderHash,
    pub producer: BlockProducer,
    pub total_input: Value,
    pub total_output: Value,
}

#[derive(Clone, Debug)]
pub enum BlockProducer {
    None,
    StakePool(PoolId),
    BftLeader(BftLeaderId),
}

#[derive(Clone, Debug)]
pub struct ExplorerTransaction {
    pub id: FragmentId,
    pub inputs: Vec<ExplorerInput>,
    pub outputs: Vec<ExplorerOutput>,
    pub certificate: Option<Certificate>,
    pub offset_in_block: u32,
    pub config_params: Option<ConfigParams>,
}

impl Default for ExplorerTransaction {
    fn default() -> Self {
        Self {
            id: Hash::zero_hash(),
            inputs: Default::default(),
            outputs: Default::default(),
            certificate: Default::default(),
            offset_in_block: Default::default(),
            config_params: Default::default(),
        }
    }
}

/// Unified Input representation for utxo and account inputs as used in the graphql API
#[derive(Clone, Debug)]
pub struct ExplorerInput {
    pub address: ExplorerAddress,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub struct ExplorerOutput {
    pub address: ExplorerAddress,
    pub value: Value,
}

#[derive(Clone)]
pub struct EpochData {
    pub first_block: HeaderHash,
    pub last_block: HeaderHash,
    pub total_blocks: u32,
}

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum ExplorerAddress {
    New(Address),
    Old(OldAddress),
}

#[derive(Clone, Debug)]
pub struct ExplorerVotePlan {
    pub id: VotePlanId,
    pub vote_start: BlockDate,
    pub vote_end: BlockDate,
    pub committee_end: BlockDate,
    pub payload_type: PayloadType,
    pub proposals: Vec<ExplorerVoteProposal>,
}

#[derive(Clone, Debug)]
pub enum ExplorerVote {
    Public(Choice),
    Private {
        proof: ProofOfCorrectVote,
        encrypted_vote: EncryptedVote,
    },
}

#[derive(Clone, Debug)]
pub struct ExplorerVoteProposal {
    pub proposal_id: ExternalProposalId,
    pub options: Options,
    pub tally: Option<ExplorerVoteTally>,
    pub votes: Hamt<ExplorerAddress, ExplorerVote>,
}

// TODO do proper vote tally
#[derive(Clone, Debug)]
pub enum ExplorerVoteTally {
    Public {
        results: Box<[Weight]>,
        options: Options,
    },
    Private {
        results: Option<Vec<Weight>>,
        options: Options,
    },
}

#[derive(Debug)]
pub struct ExplorerBlockBuildingContext<'a> {
    pub discrimination: Discrimination,
    pub prev_transactions: &'a Transactions,
    pub prev_blocks: &'a Blocks,
}

impl ExplorerBlock {
    /// Map the given `Block` to the `ExplorerBlock`, transforming all the transactions
    /// using the previous state to transform the utxo inputs to the form (Address, Amount)
    /// and mapping the account inputs to addresses with the given discrimination
    /// This function relies on the given block to be validated previously, and will panic
    /// otherwise
    #[tracing::instrument]
    pub fn resolve_from(
        block: &Block,
        context: ExplorerBlockBuildingContext,
    ) -> Result<ExplorerBlock, Error> {
        let fragments = block.contents().iter();
        let id = block.id();
        let chain_length = block.chain_length();

        let mut current_block_txs: HashMap<FragmentId, ExplorerTransaction> = HashMap::new();

        for (offset, fragment) in fragments.enumerate() {
            let fragment_id = fragment.id();
            let offset: u32 = offset.try_into().unwrap();

            let metx = match fragment {
                Fragment::Initial(config) => Some(ExplorerTransaction {
                    id: fragment_id,
                    inputs: vec![],
                    outputs: vec![],
                    certificate: None,
                    offset_in_block: offset,
                    config_params: Some(config.clone()),
                }),
                Fragment::UpdateProposal(tx) => {
                    let tx = tx.as_slice();
                    match ExplorerTransaction::from(
                        &context,
                        &fragment_id,
                        &tx,
                        Some(Certificate::UpdateProposal(tx.payload().into_payload())),
                        offset,
                        &current_block_txs,
                    ) {
                        Ok(tx) => Some(tx),
                        Err(e) => {
                            error!(error = %e, "unable to map update proposal fragment");
                            return Err(Error::ExplorerTransmuteFail);
                        }
                    }
                }
                Fragment::UpdateVote(tx) => {
                    let tx = tx.as_slice();
                    match ExplorerTransaction::from(
                        &context,
                        &fragment_id,
                        &tx,
                        Some(Certificate::UpdateVote(tx.payload().into_payload())),
                        offset,
                        &current_block_txs,
                    ) {
                        Ok(tx) => Some(tx),
                        Err(e) => {
                            error!(error = %e, "unable to map update vote fragment");
                            return Err(Error::ExplorerTransmuteFail);
                        }
                    }
                }
                Fragment::Transaction(tx) => {
                    let tx = tx.as_slice();
                    match ExplorerTransaction::from(
                        &context,
                        &fragment_id,
                        &tx,
                        None,
                        offset,
                        &current_block_txs,
                    ) {
                        Ok(tx) => Some(tx),
                        Err(e) => {
                            error!(error = %e, "unable to map tx fragment");
                            return Err(Error::ExplorerTransmuteFail);
                        }
                    }
                }
                Fragment::OwnerStakeDelegation(tx) => {
                    let tx = tx.as_slice();
                    match ExplorerTransaction::from(
                        &context,
                        &fragment_id,
                        &tx,
                        Some(Certificate::OwnerStakeDelegation(
                            tx.payload().into_payload(),
                        )),
                        offset,
                        &current_block_txs,
                    ) {
                        Ok(tx) => Some(tx),
                        Err(e) => {
                            error!(error = %e, "unable to map owner stake delegation fragment");
                            return Err(Error::ExplorerTransmuteFail);
                        }
                    }
                }
                Fragment::StakeDelegation(tx) => {
                    let tx = tx.as_slice();
                    match ExplorerTransaction::from(
                        &context,
                        &fragment_id,
                        &tx,
                        Some(Certificate::StakeDelegation(tx.payload().into_payload())),
                        offset,
                        &current_block_txs,
                    ) {
                        Ok(tx) => Some(tx),
                        Err(e) => {
                            error!(error = %e, "unable to map stake delegation fragment");
                            return Err(Error::ExplorerTransmuteFail);
                        }
                    }
                }
                Fragment::PoolRegistration(tx) => {
                    let tx = tx.as_slice();
                    match ExplorerTransaction::from(
                        &context,
                        &fragment_id,
                        &tx,
                        Some(Certificate::PoolRegistration(tx.payload().into_payload())),
                        offset,
                        &current_block_txs,
                    ) {
                        Ok(tx) => Some(tx),
                        Err(e) => {
                            error!(error = %e, "unable to map pool registration fragment");
                            return Err(Error::ExplorerTransmuteFail);
                        }
                    }
                }
                Fragment::PoolRetirement(tx) => {
                    let tx = tx.as_slice();
                    match ExplorerTransaction::from(
                        &context,
                        &fragment_id,
                        &tx,
                        Some(Certificate::PoolRetirement(tx.payload().into_payload())),
                        offset,
                        &current_block_txs,
                    ) {
                        Ok(tx) => Some(tx),
                        Err(e) => {
                            error!(error = %e, "unable to map pool retirment fragment");
                            return Err(Error::ExplorerTransmuteFail);
                        }
                    }
                }
                Fragment::PoolUpdate(tx) => {
                    let tx = tx.as_slice();
                    match ExplorerTransaction::from(
                        &context,
                        &fragment_id,
                        &tx,
                        Some(Certificate::PoolUpdate(tx.payload().into_payload())),
                        offset,
                        &current_block_txs,
                    ) {
                        Ok(tx) => Some(tx),
                        Err(e) => {
                            error!(error = %e, "unable to map pool update fragment");
                            return Err(Error::ExplorerTransmuteFail);
                        }
                    }
                }
                Fragment::VotePlan(tx) => {
                    let tx = tx.as_slice();
                    match ExplorerTransaction::from(
                        &context,
                        &fragment_id,
                        &tx,
                        Some(Certificate::VotePlan(tx.payload().into_payload())),
                        offset,
                        &current_block_txs,
                    ) {
                        Ok(tx) => Some(tx),
                        Err(e) => {
                            error!(error = %e, "unable to map vote plan fragment");
                            return Err(Error::ExplorerTransmuteFail);
                        }
                    }
                }
                Fragment::VoteCast(tx) => {
                    let tx = tx.as_slice();
                    match ExplorerTransaction::from(
                        &context,
                        &fragment_id,
                        &tx,
                        Some(Certificate::VoteCast(tx.payload().into_payload())),
                        offset,
                        &current_block_txs,
                    ) {
                        Ok(tx) => Some(tx),
                        Err(e) => {
                            error!(error = %e, "unable to map vote cast fragment");
                            return Err(Error::ExplorerTransmuteFail);
                        }
                    }
                }
                Fragment::VoteTally(tx) => {
                    let tx = tx.as_slice();
                    match ExplorerTransaction::from(
                        &context,
                        &fragment_id,
                        &tx,
                        Some(Certificate::VoteTally(tx.payload().into_payload())),
                        offset,
                        &current_block_txs,
                    ) {
                        Ok(tx) => Some(tx),
                        Err(e) => {
                            error!(error = %e, "unable to map vote fragment");
                            return Err(Error::ExplorerTransmuteFail);
                        }
                    }
                }
                Fragment::OldUtxoDeclaration(decl) => {
                    let outputs = decl
                        .addrs
                        .iter()
                        .map(|(old_address, value)| ExplorerOutput {
                            address: ExplorerAddress::Old(old_address.clone()),
                            value: *value,
                        })
                        .collect();
                    Some(ExplorerTransaction {
                        id: fragment_id,
                        inputs: vec![],
                        outputs,
                        certificate: None,
                        offset_in_block: offset,
                        config_params: None,
                    })
                }
                _ => None,
            };

            if let Some(etx) = metx {
                current_block_txs.insert(fragment_id, etx);
            }
        }

        let transactions = current_block_txs;

        let producer = match block.header().proof() {
            Proof::GenesisPraos(_proof) => {
                // Unwrap is safe in this pattern match
                BlockProducer::StakePool(block.header().get_stakepool_id().unwrap())
            }
            Proof::Bft(_proof) => {
                BlockProducer::BftLeader(block.header().get_bft_leader_id().unwrap())
            }
            Proof::None => BlockProducer::None,
        };

        let total_input = match Value::sum(
            transactions
                .values()
                .flat_map(|tx| tx.inputs.iter().map(|i| i.value)),
        ) {
            Ok(total) => total,
            Err(e) => {
                error!(error = %e, "Couldn't compute block's total input");
                return Err(Error::TxCalculationFailure);
            }
        };

        let total_output = match Value::sum(
            transactions
                .values()
                .flat_map(|tx| tx.outputs.iter().map(|o| o.value)),
        ) {
            Ok(total) => total,
            Err(e) => {
                error!(error = %e, "Couldn't compute block's total  output");
                return Err(Error::TxCalculationFailure);
            }
        };

        Ok(ExplorerBlock {
            id,
            transactions,
            chain_length,
            date: block.header().block_date(),
            parent_hash: block.parent_id(),
            producer,
            total_input,
            total_output,
        })
    }

    pub fn id(&self) -> HeaderHash {
        self.id
    }

    pub fn date(&self) -> BlockDate {
        self.date
    }

    pub fn chain_length(&self) -> ChainLength {
        self.chain_length
    }

    pub fn producer(&self) -> &BlockProducer {
        &self.producer
    }
}

impl ExplorerTransaction {
    /// Map the given AuthenticatedTransaction to the ExplorerTransaction API representation
    /// type.
    /// the fragment id is the associated to the given AuthenticatedTransaction before 'unwrapping'
    /// The discrimination is needed to get addresses from account inputs.
    /// The transactions and blocks are used to resolve utxo inputs

    pub fn from<'context, T>(
        context: &'context ExplorerBlockBuildingContext<'context>,
        id: &FragmentId,
        tx: &TransactionSlice<'_, T>,
        certificate: Option<Certificate>,
        offset_in_block: u32,
        transactions_in_current_block: &HashMap<FragmentId, ExplorerTransaction>,
    ) -> Result<ExplorerTransaction, Error> {
        let outputs = tx.outputs().iter();
        let inputs = tx.inputs().iter();
        let witnesses = tx.witnesses().iter();

        let new_outputs = outputs
            .map(|output| ExplorerOutput {
                address: ExplorerAddress::New(output.address.clone()),
                value: output.value,
            })
            .collect();

        let mut new_inputs = Vec::new();
        for i in inputs.map(|i| i.to_enum()).zip(witnesses) {
            match i {
                (InputEnum::AccountInput(_, _), Witness::Utxo(_)) => {}
                (InputEnum::AccountInput(id, value), Witness::Account(_, _)) => {
                    let kind = match id.to_single_account() {
                        Some(id) => chain_addr::Kind::Account(id.into()),
                        None => {
                            error!("cannot validate address");
                            return Err(Error::ExplorerTransmuteFail);
                        }
                    };

                    let address = ExplorerAddress::New(Address(context.discrimination, kind));
                    new_inputs.push(ExplorerInput { address, value });
                }
                (InputEnum::AccountInput(_, _), Witness::OldUtxo(_, _, _)) => {}
                (InputEnum::AccountInput(id, value), Witness::Multisig(_, _)) => {
                    let kind = chain_addr::Kind::Multisig(
                        match id.to_multi_account().as_ref().try_into() {
                            Ok(id) => id,
                            Err(e) => {
                                error!(error = %e,
                                    "multisig identifier size doesn't match address kind "
                                );
                                return Err(Error::ExplorerTransmuteFail);
                            }
                        },
                    );
                    let address = ExplorerAddress::New(Address(context.discrimination, kind));
                    new_inputs.push(ExplorerInput { address, value });
                }
                (InputEnum::UtxoInput(utxo_pointer), Witness::Utxo(_)) => {
                    let tx = utxo_pointer.transaction_id;
                    let index = utxo_pointer.output_index;

                    let output = context
                        .prev_transactions
                        .lookup(&tx)
                        .and_then(|block_id| {
                            context
                                .prev_blocks
                                .lookup(block_id)
                                .map(|block| &block.transactions[&tx].outputs[index as usize])
                        })
                        .or_else(|| {
                            match transactions_in_current_block
                                .get(&tx)
                                .map(|fragment| &fragment.outputs[index as usize])
                            {
                                Some(tx) => Some(tx),
                                None => {
                                    error!(error = %tx,"transaction not found for utxo input");
                                    None
                                }
                            }
                        });

                    match output {
                        Some(output) => new_inputs.push(ExplorerInput {
                            address: output.address.clone(),
                            value: output.value,
                        }),
                        None => return Err(Error::ExplorerTransmuteFail),
                    }
                }
                (InputEnum::UtxoInput(_), Witness::Account(_, _)) => {}
                (InputEnum::UtxoInput(_), Witness::OldUtxo(_, _, _)) => {}
                (InputEnum::UtxoInput(_), Witness::Multisig(_, _)) => {}
            }
        }

        Ok(ExplorerTransaction {
            id: *id,
            inputs: new_inputs,
            outputs: new_outputs,
            certificate,
            offset_in_block,
            config_params: None,
        })
    }

    pub fn id(&self) -> FragmentId {
        self.id
    }

    pub fn inputs(&self) -> &Vec<ExplorerInput> {
        &self.inputs
    }

    pub fn outputs(&self) -> &Vec<ExplorerOutput> {
        &self.outputs
    }
}

impl ExplorerAddress {
    pub fn to_identifier(&self) -> Option<Identifier> {
        match self {
            ExplorerAddress::New(address) => match address.kind() {
                // Single address : A simple spending key. This doesn't have any stake in the system
                chain_addr::Kind::Single(key) => Some(key.clone().into()),
                // Account address : An account key. The account is its own stake
                chain_addr::Kind::Account(key) => Some(key.clone().into()),
                // Group address : an ed25519 spending public key followed by a group public key used for staking
                chain_addr::Kind::Group(_spend, group) => Some(group.clone().into()),
                _ => None,
            },
            ExplorerAddress::Old(_) => None,
        }
    }
}
