use quickcheck::{Arbitrary, Gen};

use crate::{
    chaintypes::ConsensusType,
    config::{Block0Date, RewardParams},
    fee::{LinearFee, PerCertificateFee},
    milli::Milli,
    rewards::TaxType,
    testing::{arbitrary::NonZeroValue, ledger::ConfigBuilder},
};
use chain_addr::Discrimination;

impl Arbitrary for ConfigBuilder {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        ConfigBuilder::new(0)
            .with_rewards(NonZeroValue::arbitrary(g).0)
            .with_treasury(NonZeroValue::arbitrary(g).0)
            .with_treasury_params(TaxType::arbitrary(g))
            .with_rewards_params(RewardParams::arbitrary(g))
            .with_discrimination(Discrimination::arbitrary(g))
            .with_slot_duration(u8::arbitrary(g))
            .with_fee(LinearFee::arbitrary(g))
            .with_per_certificate_fee(PerCertificateFee::arbitrary(g))
            .with_slots_per_epoch(u32::arbitrary(g))
            .with_active_slots_coeff(Milli::arbitrary(g))
            .with_block_content_max_size(u32::arbitrary(g))
            .with_kes_update_speed(u32::arbitrary(g))
            .with_block0_date(Block0Date::arbitrary(g))
            .with_consensus_version(ConsensusType::arbitrary(g))
    }
}
