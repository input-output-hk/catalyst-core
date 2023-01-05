use chain_impl_mockchain::header::BlockDate;
use jormungandr_lib::interfaces::SettingsDto;
use thor::BlockDateGenerator;

/// Creates `BlockDateGenerator` object based on setup
pub fn from_block_or_shift(
    valid_until_fixed: Option<BlockDate>,
    valid_until_shift: Option<BlockDate>,
    block0_settings: &SettingsDto,
) -> BlockDateGenerator {
    if let Some(fixed) = valid_until_fixed {
        return BlockDateGenerator::shifted(block0_settings, fixed, false);
    }
    BlockDateGenerator::rolling(
        block0_settings,
        valid_until_shift.unwrap_or(BlockDate {
            epoch: 1,
            slot_id: 0,
        }),
        false,
    )
}
/// Creates `BlockDateGenerator` object based on settings
pub fn default_block_date_generator(block0_settings: &SettingsDto) -> BlockDateGenerator {
    BlockDateGenerator::rolling(
        block0_settings,
        BlockDate {
            epoch: 1,
            slot_id: 0,
        },
        false,
    )
}
