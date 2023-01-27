use quickcheck::{Arbitrary, Gen};

use crate::{
    account::Ledger as AccountLedger,
    block::Block,
    certificate::{MintToken, PoolId},
    chaintypes::{ChainLength, ConsensusType, ConsensusVersion, HeaderId},
    config::{Block0Date, ConfigParam, RewardParams},
    date::BlockDate,
    fee::{LinearFee, PerCertificateFee, PerVoteCertificateFee},
    fragment::{config::ConfigParams, Fragment, FragmentId},
    key::BftLeaderId,
    leadership::genesis::LeadershipData,
    ledger::{
        check::CHECK_TX_MAXIMUM_INPUTS, Error, LeadersParticipationRecord, Ledger, Pots,
        RewardsInfoParameters,
    },
    milli::Milli,
    rewards::{Ratio, TaxType},
    setting::Settings,
    stake::PoolsState,
    testing::{
        builders::{BftBlockBuilder, GenesisPraosBlockBuilder},
        data::{AddressData, AddressDataValue, LeaderPair, StakePool, Wallet},
    },
    tokens::minting_policy::MintingPolicy,
    transaction::{Output, TxBuilder},
    utxo::{Entry, Iter},
    value::Value,
    vote::CommitteeId,
};
use chain_addr::{Address, Discrimination};
use chain_crypto::*;
use chain_time::TimeEra;
use std::{
    collections::HashMap,
    num::{NonZeroU32, NonZeroU64},
};

#[derive(Clone)]
pub struct ConfigBuilder {
    slot_duration: u8,
    slots_per_epoch: u32,
    active_slots_coeff: Milli,
    discrimination: Discrimination,
    linear_fee: Option<LinearFee>,
    per_certificate_fee: Option<PerCertificateFee>,
    per_vote_certificate_fee: Option<PerVoteCertificateFee>,
    leaders: Vec<BftLeaderId>,
    committees_ids: Vec<CommitteeId>,
    rewards: Value,
    treasury: Value,
    fees_in_treasury: bool,
    treasury_params: TaxType,
    reward_params: RewardParams,
    block_content_max_size: Option<u32>,
    kes_update_speed: u32,
    block0_date: Block0Date,
    consensus_version: ConsensusVersion,
    pool_capping_ratio: Ratio,
    transaction_max_expiry_epochs: Option<u8>,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        ConfigBuilder::new()
    }
}

impl ConfigBuilder {
    pub fn new() -> Self {
        ConfigBuilder {
            slot_duration: 20,
            slots_per_epoch: 21600,
            active_slots_coeff: Milli::HALF,
            discrimination: Discrimination::Test,
            leaders: Vec::new(),
            linear_fee: None,
            per_certificate_fee: None,
            per_vote_certificate_fee: None,
            committees_ids: Vec::new(),
            pool_capping_ratio: Ratio {
                numerator: 0,
                denominator: NonZeroU64::new(1).unwrap(),
            },
            fees_in_treasury: false,
            rewards: Value(1_000_000),
            reward_params: RewardParams::Linear {
                constant: 100,
                ratio: Ratio {
                    numerator: 1,
                    denominator: NonZeroU64::new(100).unwrap(),
                },
                epoch_start: 0,
                epoch_rate: NonZeroU32::new(1).unwrap(),
            },
            treasury_params: TaxType::zero(),
            treasury: Value(1_000),
            block_content_max_size: None,
            kes_update_speed: 3600 * 12,
            block0_date: Block0Date(0),
            consensus_version: ConsensusVersion::Bft,
            transaction_max_expiry_epochs: None,
        }
    }

    pub fn with_committee_id(mut self, committee_id: CommitteeId) -> Self {
        self.committees_ids.push(committee_id);
        self
    }

    pub fn with_rewards(mut self, value: Value) -> Self {
        self.rewards = value;
        self
    }

    pub fn with_treasury(mut self, value: Value) -> Self {
        self.treasury = value;
        self
    }

    pub fn with_fees_in_treasury(mut self) -> Self {
        self.fees_in_treasury = true;
        self
    }

    pub fn with_pool_reward_participation_caping(
        mut self,
        numerator: u64,
        denominator: u64,
    ) -> Self {
        self.pool_capping_ratio = Ratio {
            numerator,
            denominator: NonZeroU64::new(denominator).unwrap(),
        };
        self
    }

    pub fn with_treasury_params(mut self, tax_type: TaxType) -> Self {
        self.treasury_params = tax_type;
        self
    }

    pub fn with_rewards_params(mut self, reward_params: RewardParams) -> Self {
        self.reward_params = reward_params;
        self
    }

    pub fn with_discrimination(mut self, discrimination: Discrimination) -> Self {
        self.discrimination = discrimination;
        self
    }

    pub fn with_slot_duration(mut self, slot_duration: u8) -> Self {
        self.slot_duration = slot_duration;
        self
    }

    pub fn with_leaders(mut self, leaders: &[BftLeaderId]) -> Self {
        self.leaders.extend(leaders.iter().cloned());
        self
    }

    pub fn with_fee(mut self, linear_fee: LinearFee) -> Self {
        self.linear_fee = Some(linear_fee);
        self
    }

    pub fn with_per_certificate_fee(mut self, per_certificate_fee: PerCertificateFee) -> Self {
        self.per_certificate_fee = Some(per_certificate_fee);
        self
    }

    pub fn with_per_vote_certificate_fee(
        mut self,
        per_vote_certificate_fee: PerVoteCertificateFee,
    ) -> Self {
        self.per_vote_certificate_fee = Some(per_vote_certificate_fee);
        self
    }

    pub fn with_slots_per_epoch(mut self, slots_per_epoch: u32) -> Self {
        self.slots_per_epoch = slots_per_epoch;
        self
    }

    pub fn with_active_slots_coeff(mut self, active_slots_coeff: Milli) -> Self {
        self.active_slots_coeff = active_slots_coeff;
        self
    }

    pub fn with_block_content_max_size(mut self, block_content_max_size: u32) -> Self {
        self.block_content_max_size = Some(block_content_max_size);
        self
    }

    pub fn with_kes_update_speed(mut self, kes_update_speed: u32) -> Self {
        self.kes_update_speed = kes_update_speed;
        self
    }

    pub fn with_block0_date(mut self, block0_date: Block0Date) -> Self {
        self.block0_date = block0_date;
        self
    }

    pub fn with_consensus_version(mut self, consensus_version: ConsensusType) -> Self {
        self.consensus_version = consensus_version;
        self
    }

    pub fn with_transaction_max_expiry_epochs(mut self, n_epochs: u8) -> Self {
        self.transaction_max_expiry_epochs = Some(n_epochs);
        self
    }

    fn create_single_bft_leader() -> BftLeaderId {
        let leader_prv_key: SecretKey<Ed25519Extended> = SecretKey::generate(rand_core::OsRng);
        let leader_pub_key = leader_prv_key.to_public();
        leader_pub_key.into()
    }

    pub fn normalize(&mut self) {
        // TODO remove rng: make this creation deterministic
        if self.leaders.is_empty() {
            self.leaders.push(Self::create_single_bft_leader());
        }
    }

    pub fn build(self) -> ConfigParams {
        let mut ie = ConfigParams::new();
        ie.push(ConfigParam::Discrimination(self.discrimination));
        ie.push(ConfigParam::ConsensusVersion(self.consensus_version));

        for leader_id in self.leaders.iter().cloned() {
            ie.push(ConfigParam::AddBftLeader(leader_id));
        }

        ie.push(ConfigParam::RewardPot(self.rewards));
        ie.push(ConfigParam::TreasuryAdd(self.treasury));
        ie.push(ConfigParam::TreasuryParams(self.treasury_params));
        ie.push(ConfigParam::RewardParams(self.reward_params.clone()));
        ie.push(ConfigParam::FeesInTreasury(self.fees_in_treasury));

        if self.pool_capping_ratio.numerator >= 1 {
            ie.push(ConfigParam::PoolRewardParticipationCapping((
                NonZeroU32::new(self.pool_capping_ratio.numerator as u32).unwrap(),
                NonZeroU32::new(self.pool_capping_ratio.numerator as u32).unwrap(),
            )));
        }

        if let Some(linear_fee) = self.linear_fee {
            ie.push(ConfigParam::LinearFee(linear_fee));
        }

        if let Some(block_content_max_size) = self.block_content_max_size {
            ie.push(ConfigParam::BlockContentMaxSize(block_content_max_size));
        }

        if let Some(per_certificate_fee) = self.per_certificate_fee {
            ie.push(ConfigParam::PerCertificateFees(per_certificate_fee));
        }

        if let Some(per_vote_certificate_fee) = self.per_vote_certificate_fee {
            ie.push(ConfigParam::PerVoteCertificateFees(
                per_vote_certificate_fee,
            ));
        }

        if let Some(transaction_max_expiry_epochs) = self.transaction_max_expiry_epochs {
            ie.push(ConfigParam::TransactionMaxExpiryEpochs(
                transaction_max_expiry_epochs,
            ));
        }

        for committee_id in self.committees_ids {
            ie.push(ConfigParam::AddCommitteeId(committee_id));
        }

        ie.push(ConfigParam::Block0Date(self.block0_date));
        ie.push(ConfigParam::SlotDuration(self.slot_duration));
        ie.push(ConfigParam::ConsensusGenesisPraosActiveSlotsCoeff(
            self.active_slots_coeff,
        ));
        ie.push(ConfigParam::SlotsPerEpoch(self.slots_per_epoch));
        ie.push(ConfigParam::KesUpdateSpeed(self.kes_update_speed));
        ie
    }
}

#[derive(Clone)]
pub struct LedgerBuilder {
    cfg_builder: ConfigBuilder,
    cfg_params: ConfigParams,
    fragments: Vec<Fragment>,
    certs: Vec<Fragment>,
    faucets: Vec<AddressDataValue>,
    utxo_declaration: Vec<UtxoDeclaration>,
}

pub type UtxoDeclaration = Output<Address>;

#[derive(Clone, Debug)]
pub struct UtxoDb {
    db: HashMap<(FragmentId, u8), UtxoDeclaration>,
}

impl UtxoDb {
    pub fn find_fragments(&self, decl: &UtxoDeclaration) -> Vec<(FragmentId, u8)> {
        self.db
            .iter()
            .filter_map(|(k, v)| if v == decl { Some(k) } else { None })
            .copied()
            .collect()
    }

    pub fn get(&self, key: &(FragmentId, u8)) -> Option<&UtxoDeclaration> {
        self.db.get(key)
    }
}

impl LedgerBuilder {
    pub fn from_config(mut cfg_builder: ConfigBuilder) -> Self {
        cfg_builder.normalize();
        let cfg_params = cfg_builder.clone().build();
        Self {
            cfg_builder,
            cfg_params,
            faucets: Vec::new(),
            utxo_declaration: Vec::new(),
            fragments: Vec::new(),
            certs: Vec::new(),
        }
    }

    pub fn fragment(mut self, f: Fragment) -> Self {
        self.fragments.push(f);
        self
    }

    pub fn fragments(mut self, f: &[Fragment]) -> Self {
        self.fragments.extend_from_slice(f);
        self
    }

    pub fn certs(mut self, f: &[Fragment]) -> Self {
        self.certs.extend_from_slice(f);
        self
    }

    // add a fragment that pre-fill the address with a specific value at ledger start
    pub fn prefill_address(self, address: Address, value: Value) -> Self {
        self.prefill_output(Output { address, value })
    }

    pub fn prefill_output(self, output: Output<Address>) -> Self {
        let tx = TxBuilder::new()
            .set_nopayload()
            .set_expiry_date(BlockDate::first().next_epoch())
            .set_ios(&[], &[output])
            .set_witnesses(&[])
            .set_payload_auth(&());
        self.fragment(Fragment::Transaction(tx))
    }

    pub fn prefill_outputs(mut self, outputs: &[Output<Address>]) -> Self {
        for outputs_chunk in outputs.chunks(CHECK_TX_MAXIMUM_INPUTS.into()) {
            let tx = TxBuilder::new()
                .set_nopayload()
                .set_expiry_date(BlockDate::first().next_epoch())
                .set_ios(&[], outputs_chunk)
                .set_witnesses(&[])
                .set_payload_auth(&());
            self = self.fragment(Fragment::Transaction(tx));
        }

        self
    }

    pub fn mint_tokens(mut self) -> Self {
        for faucet in &self.faucets {
            for (token, value) in &faucet.tokens {
                let tx = TxBuilder::new()
                    .set_payload(&MintToken {
                        name: token.clone(),
                        policy: MintingPolicy::new(),
                        to: faucet.to_id(),
                        value: *value,
                    })
                    .set_expiry_date(BlockDate::first().next_epoch())
                    .set_ios(&[], &[])
                    .set_witnesses(&[])
                    .set_payload_auth(&());

                self.fragments.push(Fragment::MintToken(tx));
            }
        }

        self
    }

    pub fn faucet_value(mut self, value: Value) -> Self {
        self.faucets.push(AddressDataValue::account(
            self.cfg_builder.discrimination,
            value,
        ));
        self
    }

    pub fn initial_fund(mut self, fund: &AddressDataValue) -> Self {
        if fund.is_utxo() {
            self = self.utxos(&[fund.make_output()]);
        } else {
            self = self.faucet(fund);
        }
        self
    }

    pub fn initial_funds(mut self, funds: &[AddressDataValue]) -> Self {
        for fund in funds {
            self = self.initial_fund(fund);
        }
        self
    }

    pub fn faucet(mut self, faucet: &AddressDataValue) -> Self {
        self.faucets.push(faucet.clone());
        self
    }

    pub fn faucets_wallets(mut self, faucets: Vec<&Wallet>) -> Self {
        self.faucets
            .extend(faucets.iter().cloned().map(|x| x.as_account()));
        self
    }

    pub fn faucets(mut self, faucets: &[AddressDataValue]) -> Self {
        self.faucets.extend(faucets.iter().cloned());
        self
    }

    pub fn utxos(mut self, decls: &[UtxoDeclaration]) -> Self {
        self.utxo_declaration.extend_from_slice(decls);
        self
    }

    pub fn build(mut self) -> Result<TestLedger, Error> {
        let block0_hash = HeaderId::hash_bytes(&[1, 2, 3]);
        let outputs: Vec<Output<Address>> = self.faucets.iter().map(|x| x.make_output()).collect();
        self = self.prefill_outputs(&outputs);
        self = self.mint_tokens();

        let utxodb = if !self.utxo_declaration.is_empty() {
            let mut db = HashMap::new();

            // TODO subdivide utxo_declaration in group of 254 elements
            // and repeatdly create fragment
            assert!(self.utxo_declaration.len() < 254);
            let group = self.utxo_declaration;
            {
                let tx = TxBuilder::new()
                    .set_nopayload()
                    .set_expiry_date(BlockDate::first().next_epoch())
                    .set_ios(&[], &group)
                    .set_witnesses(&[])
                    .set_payload_auth(&());
                let fragment = Fragment::Transaction(tx);
                let fragment_id = fragment.hash();

                for (idx, o) in group.iter().enumerate() {
                    let m = db.insert((fragment_id, idx as u8), o.clone());
                    assert!(m.is_none());
                }

                self.fragments.push(fragment);
            }
            UtxoDb { db }
        } else {
            UtxoDb { db: HashMap::new() }
        };

        let cfg = self.cfg_params.clone();

        let mut fragments = vec![Fragment::Initial(self.cfg_params)];
        fragments.extend_from_slice(&self.fragments);
        fragments.extend_from_slice(&self.certs);

        let faucets = self.faucets;
        Ledger::new(block0_hash, &fragments).map(|ledger| TestLedger {
            cfg,
            faucets,
            ledger,
            block0_hash,
            utxodb,
        })
    }
}
#[derive(Clone, Debug)]
pub struct TestLedger {
    pub block0_hash: HeaderId,
    pub cfg: ConfigParams,
    pub faucets: Vec<AddressDataValue>,
    pub ledger: Ledger,
    pub utxodb: UtxoDb,
}

impl TestLedger {
    pub fn apply_transaction(&mut self, fragment: Fragment) -> Result<(), Error> {
        let fragment_id = fragment.hash();
        match fragment {
            Fragment::Transaction(tx) => {
                match self
                    .ledger
                    .clone()
                    .apply_transaction(&fragment_id, &tx.as_slice())
                {
                    Err(err) => Err(err),
                    Ok((ledger, _)) => {
                        // TODO more bookkeeping for accounts and utxos
                        self.ledger = ledger;
                        Ok(())
                    }
                }
            }
            _ => panic!("test ledger apply transaction only supports transaction type for now"),
        }
    }

    pub fn apply_fragment(&mut self, fragment: &Fragment, date: BlockDate) -> Result<(), Error> {
        self.ledger = self.ledger.clone().apply_fragment(fragment, date)?;
        Ok(())
    }

    pub fn apply_block(&mut self, block: Block) -> Result<(), Error> {
        let header_meta = block.header().get_content_eval_context();
        self.ledger = self
            .ledger
            .clone()
            .apply_block(block.contents(), &header_meta)?;
        Ok(())
    }

    pub fn apply_protocol_changes(&mut self) -> Result<(), Error> {
        self.ledger = self.ledger.apply_protocol_changes()?;
        Ok(())
    }

    pub fn total_funds(&self) -> Value {
        self.ledger
            .get_total_value()
            .expect("total ledger funds are too big")
    }

    pub fn find_utxo_for_address<'a>(
        &'a self,
        address_data: &AddressData,
    ) -> Option<Entry<'a, Address>> {
        self.utxos()
            .find(|x| x.output.address == address_data.address)
    }

    pub fn accounts(&self) -> &AccountLedger {
        self.ledger.accounts()
    }

    pub fn block0_hash(&self) -> &HeaderId {
        &self.block0_hash
    }

    pub fn faucets(&self) -> Vec<AddressDataValue> {
        self.faucets.clone()
    }

    pub fn utxos(&self) -> Iter<'_, Address> {
        self.ledger.utxos()
    }

    pub fn fee(&self) -> LinearFee {
        self.ledger.settings.linear_fees.clone()
    }

    pub fn chain_length(&self) -> ChainLength {
        self.ledger.chain_length()
    }

    pub fn era(&self) -> &TimeEra {
        self.ledger.era()
    }

    pub fn delegation(&self) -> PoolsState {
        self.ledger.delegation().clone()
    }

    pub fn date(&self) -> BlockDate {
        self.ledger.date()
    }

    // use it only for negative testing since it introduce bad state in ledger
    pub fn set_date(&mut self, date: BlockDate) {
        self.ledger.date = date;
    }

    pub fn leaders_log(&self) -> LeadersParticipationRecord {
        self.ledger.leaders_log.clone()
    }

    pub fn leaders_log_for(&self, pool_id: &PoolId) -> u32 {
        *self
            .leaders_log()
            .iter()
            .find(|record| *record.0 == *pool_id)
            .unwrap()
            .1
    }

    // use it only for negative testing since it introduce bad state in ledger
    pub fn increase_leader_log(&mut self, pool_id: &PoolId) {
        self.ledger.leaders_log.increase_for(pool_id);
    }

    pub fn distribute_rewards(&mut self) -> Result<(), Error> {
        match self.ledger.distribute_rewards(
            &self.ledger.get_stake_distribution(),
            RewardsInfoParameters::default(),
        ) {
            Err(err) => Err(err),
            Ok((ledger, _)) => {
                self.ledger = ledger;
                Ok(())
            }
        }
    }

    pub fn forge_empty_block(&self, stake_pool: &StakePool) -> Block {
        self.forge_praos_block_with_fragments(stake_pool, Vec::new())
    }

    pub fn apply_empty_praos_block(&mut self, stake_pool: &StakePool) -> Result<(), Error> {
        self.apply_praos_block(stake_pool, vec![])
    }

    pub fn settings(&self) -> &Settings {
        self.ledger.settings()
    }

    pub fn apply_praos_block(
        &mut self,
        stake_pool: &StakePool,
        fragments: Vec<Fragment>,
    ) -> Result<(), Error> {
        let block = self.forge_praos_block_with_fragments(stake_pool, fragments);
        self.apply_block(block)
    }

    pub fn forge_praos_block_with_fragments(
        &self,
        stake_pool: &StakePool,
        fragments: Vec<Fragment>,
    ) -> Block {
        GenesisPraosBlockBuilder::new()
            .with_date(self.date())
            .with_fragments(fragments)
            .with_chain_length(self.ledger.chain_length())
            .with_parent_id(self.block0_hash)
            .build(stake_pool, self.ledger.era())
    }

    pub fn apply_empty_bft_block(&mut self, leader_pair: &LeaderPair) -> Result<(), Error> {
        let block = self.forge_bft_block_with_fragments(leader_pair, self.date(), Vec::new());
        self.apply_block(block)
    }

    pub fn apply_empty_bft_block_with_date(
        &mut self,
        leader_pair: &LeaderPair,
        block_date: BlockDate,
    ) -> Result<(), Error> {
        let block = self.forge_bft_block_with_fragments(leader_pair, block_date, Vec::new());
        self.apply_block(block)
    }

    pub fn forge_bft_block_with_fragments(
        &self,
        bft_leader: &LeaderPair,
        block_date: BlockDate,
        fragments: Vec<Fragment>,
    ) -> Block {
        BftBlockBuilder::new()
            .with_date(block_date)
            .with_fragments(fragments)
            .with_chain_length(self.ledger.chain_length().increase())
            .with_parent_id(self.block0_hash)
            .build(bft_leader, self.ledger.era())
    }

    pub fn forward_date(&mut self) {
        self.ledger.date = self.ledger.date.next(self.ledger.era());
    }

    pub fn can_distribute_reward(&self) -> bool {
        self.ledger.can_distribute_reward()
    }

    pub fn fast_forward_to(&mut self, date: BlockDate) {
        self.set_date(date);
    }

    pub fn fire_leadership_event(
        &mut self,
        stake_pools: Vec<StakePool>,
        fragments: Vec<Fragment>,
    ) -> Result<bool, Error> {
        let selection = LeadershipData::new(
            self.date().epoch,
            self.ledger.get_stake_distribution(),
            self.ledger.delegation.clone(),
            self.ledger.settings.consensus_nonce.clone(),
            self.ledger.settings.active_slots_coeff,
        );
        for stake_pool in stake_pools {
            if selection
                .leader(
                    &stake_pool.id(),
                    stake_pool.vrf().private_key(),
                    self.ledger.date(),
                )
                .expect("cannot calculate leader")
                .is_some()
            {
                self.apply_praos_block(&stake_pool, fragments)?;
                return Ok(true);
            }
        }
        self.forward_date();
        Ok(false)
    }

    pub fn pots(&self) -> Pots {
        self.ledger.pots.clone()
    }
}

impl From<TestLedger> for Ledger {
    fn from(from: TestLedger) -> Self {
        from.ledger
    }
}

impl Arbitrary for TestLedger {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        LedgerBuilder::arbitrary(g).build().unwrap()
    }
}

impl Arbitrary for Ledger {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        TestLedger::arbitrary(g).into()
    }
}
