use crate::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeStatsDto {
    pub version: String,
    pub state: NodeState,
    #[serde(flatten)]
    pub stats: Option<NodeStats>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NodeStats {
    pub block_recv_cnt: u64,
    pub last_block_content_size: u32,
    pub last_block_date: Option<String>,
    pub last_block_fees: u64,
    pub last_block_hash: Option<String>,
    pub last_block_height: Option<String>,
    pub last_block_sum: u64,
    pub last_block_time: Option<SystemTime>,
    pub last_block_tx: u64,
    pub last_received_block_time: Option<SystemTime>,
    pub block_content_size_avg: f64,
    pub peer_available_cnt: usize,
    pub peer_connected_cnt: usize,
    pub peer_quarantined_cnt: usize,
    pub peer_total_cnt: usize,
    pub tx_recv_cnt: u64,
    pub mempool_usage_ratio: f64,
    pub mempool_tx_count: u64,
    pub tx_rejected_cnt: u64,
    pub votes_cast: u64,
    pub uptime: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum NodeState {
    StartingRestServer,
    PreparingStorage,
    PreparingBlock0,
    Bootstrapping,
    StartingWorkers,
    Running,
}
