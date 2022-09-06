use crate::{db::Db, model::TestnetMagic};

pub fn calculate_snapshot(
    db: &Db,
    slot_no: Option<SlotNo>,
    testnet_magic: Option<TestnetMagic>,
) -> Result<Vec<Output>> {
}
