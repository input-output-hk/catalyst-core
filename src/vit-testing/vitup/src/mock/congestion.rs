use crate::mock::mock_state::MockState;
use jormungandr_lib::interfaces::{BlockDate, FragmentStatus};
use vit_servicing_station_tests::common::data::Snapshot;

#[derive(Debug, Clone)]
pub struct NetworkCongestion {
    mode: NetworkCongestionMode,
    is_private: bool,
}

const PRIVATE_FRAGMENT_SIZE: u32 = 312;
const PUBLIC_FRAGMENT_SIZE: u32 = 176;

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

    fn pending_fragments_total_size(&self, pending_fragments_count: usize) -> u32 {
        if self.is_private {
            PRIVATE_FRAGMENT_SIZE * pending_fragments_count as u32
        } else {
            PUBLIC_FRAGMENT_SIZE * pending_fragments_count as u32
        }
    }

    fn jammed_percentage(&self, percentage: f32, mock_state: &MockState) -> NetworkCongestionData {
        let block_max_size = mock_state.ledger().settings().block_content_max_size;
        let max_fragment_count = block_max_size / {
            if self.is_private {
                PRIVATE_FRAGMENT_SIZE
            } else {
                PUBLIC_FRAGMENT_SIZE
            }
        };
        NetworkCongestionData {
            received_fragments_count: (max_fragment_count as f32 * percentage) as usize
                * mock_state.ledger().absolute_slot_count() as usize,
            pending_fragments_count: (max_fragment_count as f32 * percentage) as usize,
            rejected_fragments_count: (max_fragment_count as f32 * percentage / 2.0) as usize
                * mock_state.ledger().absolute_slot_count() as usize,
            pending_fragments_total_size: (block_max_size as f32 * percentage) as u64,
            block_content_size_avg: (block_max_size as f32 * percentage) as u64,
        }
    }

    pub fn calculate(&self, mock_state: &MockState) -> NetworkCongestionData {
        let current_blockchain_age = mock_state.ledger().current_blockchain_age();

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
                    pending_fragments_count,
                    rejected_fragments_count: mock_state
                        .ledger()
                        .fragment_logs()
                        .iter()
                        .filter(|x| x.is_rejected())
                        .count(),
                    pending_fragments_total_size: self
                        .pending_fragments_total_size(pending_fragments_count)
                        as u64,
                    block_content_size_avg: (0..=2).fold(0u64, |sum, val| {
                        let block_date = shift_slot_back(current_blockchain_age, val);
                        sum + mock_state
                            .ledger()
                            .fragment_logs()
                            .iter()
                            .filter(|f| {
                                if let FragmentStatus::InABlock { date, block: _ } = f.status() {
                                    *date == block_date
                                } else {
                                    false
                                }
                            })
                            .count() as u64
                    }),
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
    pub pending_fragments_count: usize,
    pub rejected_fragments_count: usize,
    pub pending_fragments_total_size: u64,
    pub block_content_size_avg: u64,
}
