use crate::mode::mock::mock_state::MockState;
use jormungandr_lib::interfaces::{BlockDate, FragmentStatus};
use vit_servicing_station_tests::common::data::Snapshot;

#[derive(Debug, Clone)]
pub struct NetworkCongestion {
    mode: NetworkCongestionMode,
    is_private: bool,
}

const PRIVATE_FRAGMENT_SIZE: usize = 312;
const PUBLIC_FRAGMENT_SIZE: usize = 176;

impl NetworkCongestion {
    pub fn new(snapshot: &Snapshot) -> Self {
        Self {
            mode: NetworkCongestionMode::Disabled,
            is_private: snapshot.funds()[0].chain_vote_plans[0].chain_voteplan_payload == "private",
        }
    }

    pub fn set_mode(&mut self, mode: NetworkCongestionMode) {
        self.mode = mode;
    }

    fn fragments_total_size(&self, pending_fragments_count: usize) -> usize {
        if self.is_private {
            PRIVATE_FRAGMENT_SIZE * pending_fragments_count
        } else {
            PUBLIC_FRAGMENT_SIZE * pending_fragments_count
        }
    }

    fn jammed_percentage(&self, percentage: f64, mock_state: &MockState) -> NetworkCongestionData {
        let block_max_size = mock_state.ledger().settings().block_content_max_size;
        let max_fragment_count = block_max_size / {
            if self.is_private {
                PRIVATE_FRAGMENT_SIZE
            } else {
                PUBLIC_FRAGMENT_SIZE
            }
        } as u32;
        NetworkCongestionData {
            received_fragments_count: (max_fragment_count as f64 * percentage) as usize
                * mock_state.ledger().absolute_slot_count() as usize,
            mempool_usage_ratio: percentage,
            rejected_fragments_count: (max_fragment_count as f64 * percentage / 2.0) as usize
                * mock_state.ledger().absolute_slot_count() as usize,
            mempool_tx_count: (max_fragment_count as f64 * percentage) as u64,
            block_content_size_avg: percentage,
        }
    }

    pub fn calculate(&self, mock_state: &MockState) -> NetworkCongestionData {
        let current_blockchain_age = mock_state.ledger().current_blockchain_age();
        let block_max_size = mock_state.ledger().settings().block_content_max_size as usize;

        match self.mode {
            NetworkCongestionMode::Disabled => {
                let pending_fragments_count = mock_state
                    .ledger()
                    .fragment_logs()
                    .iter()
                    .filter(|x| x.is_pending())
                    .count();

                NetworkCongestionData {
                    received_fragments_count: mock_state.ledger().received_fragments().len(),
                    mempool_usage_ratio: self.fragments_total_size(pending_fragments_count) as f64
                        / block_max_size as f64,
                    rejected_fragments_count: mock_state
                        .ledger()
                        .fragment_logs()
                        .iter()
                        .filter(|x| x.is_rejected())
                        .count(),
                    mempool_tx_count: pending_fragments_count as u64,
                    block_content_size_avg: self.fragments_total_size((0..=2).fold(
                        0usize,
                        |sum, val| {
                            let block_date = shift_slot_back(current_blockchain_age, val);
                            sum + mock_state
                                .ledger()
                                .fragment_logs()
                                .iter()
                                .filter(|f| {
                                    if let FragmentStatus::InABlock { date, block: _ } = f.status()
                                    {
                                        *date == block_date
                                    } else {
                                        false
                                    }
                                })
                                .count()
                        },
                    )) as f64
                        / block_max_size as f64,
                }
            }
            NetworkCongestionMode::Normal => self.jammed_percentage(0.15, mock_state),
            NetworkCongestionMode::Jammed => self.jammed_percentage(1.0, mock_state),
            NetworkCongestionMode::Moderate => self.jammed_percentage(0.50, mock_state),
        }
    }
}

//TODO move to jormungandr lib BlockDate struct
pub fn shift_slot_back(block_date: BlockDate, slot_shift: u32) -> BlockDate {
    let mut block_date: chain_impl_mockchain::block::BlockDate = block_date.into();
    for _ in 0..slot_shift {
        if block_date.slot_id == 0 {
            if block_date.epoch != 0 {
                block_date.epoch -= 1;
            }
        } else {
            block_date.slot_id -= 1;
        }
    }
    block_date.into()
}

#[derive(Copy, Clone, Debug)]
pub enum NetworkCongestionMode {
    Jammed,
    Moderate,
    Normal,
    Disabled,
}

#[derive(Copy, Clone, Debug)]
pub struct NetworkCongestionData {
    pub received_fragments_count: usize,
    pub mempool_usage_ratio: f64,
    pub rejected_fragments_count: usize,
    pub mempool_tx_count: u64,
    pub block_content_size_avg: f64,
}
