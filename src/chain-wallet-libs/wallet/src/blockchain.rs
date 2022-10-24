use chain_addr::Discrimination;
use chain_impl_mockchain::{
    block::Block,
    config::{Block0Date, ConfigParam},
    fee::{FeeAlgorithm as _, LinearFee},
    fragment::Fragment,
    header::HeaderId,
    ledger::{Block0Error, Error, Ledger},
    transaction::Input,
};
use chain_time::TimeEra;

#[derive(Clone)]
pub struct Settings {
    pub fees: LinearFee,
    pub discrimination: Discrimination,
    pub block0_initial_hash: HeaderId,
    pub block0_date: Block0Date,
    pub slot_duration: u8,
    pub time_era: TimeEra,
    pub transaction_max_expiry_epochs: u8,
}

impl Settings {
    pub fn new(block: &Block) -> Result<Self, Error> {
        let header_id = block.header().id();
        let ledger = Ledger::new(header_id, block.contents().iter())?;

        let static_parameters = ledger.get_static_parameters().clone();
        let parameters = ledger.settings();

        // TODO: I think there is a bug in Ledger::new(), as it doesn't set the slot_duration in
        // the Settings.
        // This doesn't seem to matter for jormungandr anyway, because it gets the initial value
        // directly from the block0, and then just doesn't use the field in the settings anymore.
        // For now, just get the setting directly from the block0 here too, but should probably be
        // fixed in Ledger::new (or at least checked).
        let mut slot_duration = None;

        for fragment in block.contents().iter() {
            if let Fragment::Initial(initials) = fragment {
                for initial in initials.iter() {
                    match initial {
                        ConfigParam::Block0Date(_) => {}
                        ConfigParam::Discrimination(_) => {}
                        ConfigParam::ConsensusVersion(_) => {}
                        ConfigParam::SlotsPerEpoch(_) => {}
                        ConfigParam::SlotDuration(sd) => {
                            slot_duration.replace(sd);
                        }
                        ConfigParam::EpochStabilityDepth(_) => {}
                        ConfigParam::ConsensusGenesisPraosActiveSlotsCoeff(_) => {}
                        ConfigParam::BlockContentMaxSize(_) => {}
                        ConfigParam::AddBftLeader(_) => {}
                        ConfigParam::RemoveBftLeader(_) => {}
                        ConfigParam::LinearFee(_) => {}
                        ConfigParam::ProposalExpiration(_) => {}
                        ConfigParam::KesUpdateSpeed(_) => {}
                        ConfigParam::TreasuryAdd(_) => {}
                        ConfigParam::TreasuryParams(_) => {}
                        ConfigParam::RewardPot(_) => {}
                        ConfigParam::RewardParams(_) => {}
                        ConfigParam::PerCertificateFees(_) => {}
                        ConfigParam::FeesInTreasury(_) => {}
                        ConfigParam::RewardLimitNone => {}
                        ConfigParam::RewardLimitByAbsoluteStake(_) => {}
                        ConfigParam::PoolRewardParticipationCapping(_) => {}
                        ConfigParam::AddCommitteeId(_) => {}
                        ConfigParam::RemoveCommitteeId(_) => {}
                        ConfigParam::PerVoteCertificateFees(_) => {}
                        ConfigParam::TransactionMaxExpiryEpochs(_) => {}
                        ConfigParam::EvmConfiguration(_) => unimplemented!(),
                        ConfigParam::EvmEnvironment(_) => unimplemented!(),
                    }
                }
            }
        }

        Ok(Self {
            fees: parameters.linear_fees.clone(),
            discrimination: static_parameters.discrimination,
            block0_initial_hash: static_parameters.block0_initial_hash,
            block0_date: static_parameters.block0_start_time,
            slot_duration: *slot_duration
                .ok_or(Error::Block0(Block0Error::InitialMessageNoSlotDuration))?,
            time_era: ledger.era().clone(),
            transaction_max_expiry_epochs: ledger.settings().transaction_max_expiry_epochs,
        })
    }

    /// convenient function to check if a given input
    /// is covering at least its own input fees for a given transaction
    pub fn is_input_worth(&self, input: &Input) -> bool {
        let value = input.value();
        let minimal_value = self.fees.fees_for_inputs_outputs(1, 0);

        value > minimal_value
    }

    pub fn discrimination(&self) -> Discrimination {
        self.discrimination
    }
}
