use crate::db::{Meta, Progress};
use crate::rest::v0::errors::HandleError;
use crate::rest::v0::DataProvider;
use crate::{BehindDuration, TransactionConfirmation};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use cardano_serialization_lib::utils::from_bignum;
use jormungandr_lib::interfaces::BlockDate;
use mainnet_lib::{BlockDateFromCardanoAbsoluteSlotNo, InMemoryDbSync, Ledger};
use std::time::SystemTime;

pub struct Provider {
    db_sync: InMemoryDbSync,
    ledger: Ledger,
    meta_info: Meta,
    behind_duration: BehindDuration,
    progress: Progress,
}

impl Provider {
    pub fn from_ledger_and_db_sync(ledger: Ledger, db_sync: InMemoryDbSync) -> Self {
        Provider {
            ledger,
            db_sync,
            ..Default::default()
        }
    }

    pub fn db_sync_content(&self) -> Result<String, serde_json::Error> {
        self.db_sync.try_as_string()
    }

    pub fn db_sync_mut(&mut self) -> &mut InMemoryDbSync {
        &mut self.db_sync
    }

    pub fn ledger_mut(&mut self) -> &mut Ledger {
        &mut self.ledger
    }
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            db_sync: InMemoryDbSync::default(),
            ledger: Ledger::default(),
            meta_info: Meta {
                id: 0,
                start_time: SystemTime::now(),
                version: "Mocked".to_string(),
                network_name: "Mocked".to_string(),
            },
            behind_duration: BehindDuration {
                behind_by: SystemTime::UNIX_EPOCH,
            },
            progress: Progress {
                sync_percentage: BigDecimal::parse_bytes(b"99", 100).unwrap(),
            },
        }
    }
}

#[async_trait]
impl DataProvider for Provider {
    async fn get_meta_info(&self) -> Result<Vec<Meta>, HandleError> {
        Ok(vec![self.meta_info.clone()])
    }

    async fn get_interval_behind_now(&self) -> Result<BehindDuration, HandleError> {
        Ok(self.behind_duration.clone())
    }

    async fn get_sync_progress(&self) -> Result<Progress, HandleError> {
        Ok(self.progress.clone())
    }

    async fn get_tx_by_hash(
        &self,
        hash: String,
    ) -> Result<Vec<TransactionConfirmation>, HandleError> {
        Ok(self
            .db_sync
            .transaction_by_hash(&hash)
            .into_iter()
            .map(|(block, _tx)| {
                let absolute_slot = block
                    .as_ref()
                    .map(|block| from_bignum(&block.header().header_body().slot_bignum()));
                let block_date = absolute_slot.map(BlockDate::from_absolute_slot_no);
                TransactionConfirmation {
                    epoch_no: block_date.map(|block_date| block_date.epoch().try_into().unwrap()),
                    slot_no: block_date.map(|block_date| block_date.slot().into()),
                    absolute_slot: absolute_slot
                        .map(|absolute_slot| absolute_slot.try_into().unwrap()),
                    block_no: block.as_ref().map(|block| {
                        block
                            .header()
                            .header_body()
                            .block_number()
                            .try_into()
                            .unwrap()
                    }),
                }
            })
            .collect())
    }
}
