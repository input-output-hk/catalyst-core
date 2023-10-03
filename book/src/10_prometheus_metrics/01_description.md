# `jormungandr` Prometheus Metrics

`jormungadr` uses Prometheus metrics to gather information about the node at runtime.

## Fragment Mempool Process

As the node receives fragments, they are inserted into the fragment mempool, and propagated into the peer network.

### `txRecvCnt`

    >> tx_recv_cnt: IntCounter,

Total number of tx inserted and propagated by the mempool at each loop in the process.

### `txRejectedCnt`

    >> tx_rejected_cnt: IntCounter,

Total number of tx rejected by the mempool at each loop in the process.

### `mempoolTxCount`

    >> mempool_tx_count: UIntGauge,

Total number of tx in the mempool for a given block

### `mempoolUsageRatio`

    >> mempool_usage_ratio: Gauge,

Mempool usage ratio for a given block

## Topology Process

As the node connects to peers, the network topology allows for gossip and p2p communication. Nodes can join or leave the network.

### `peerConnectedCnt`

    >> peer_connected_cnt: UIntGauge,

The total number of connected peers.

### `peerQuarantinedCnt`

    >> peer_quarantined_cnt: UIntGauge,

The total number of quarantined peers.

### `peerAvailableCnt`

    >> peer_available_cnt: UIntGauge,

The total number of available peers.

### `peerTotalCnt`

    >> peer_total_cnt: UIntGauge,

The total number of peers.

## Blockchain Process

Each node receives blocks streamed from the network which are processed in order to create a new block tip.

### `blockRecvCnt`

    >> block_recv_cnt: IntCounter,

This is the total number of blocks streamed from the network that will be processed at each loop in the process.

## Blockchain Tip-Block Process

As the node sets the tip-block, this happens when the node is started and during the block minting process, these metrics are updated.

### `votesCasted`

    >> votes_casted_cnt: IntCounter,

The total number accepted `VoteCast` fragments. Metric is incremented by the total number of valid `VoteCast` fragments
in the block tip.

### `lastBlockTx`

    >> // Total number of tx for a given block
    >> block_tx_count: IntCounter,

The total number of valid transaction fragments in the block tip.

### `lastBlockInputTime` <--- **misnomer**

    >> block_input_sum: UIntGauge,

The total sum of transaction input values in the block tip. The `tx.total_input()` is added for every fragment.

### `lastBlockSum`

    >> block_fee_sum: UIntGauge,

The total sum of transaction output values (fees) in the block tip. The `tx.total_output()` is added for every fragment.

### `lastBlockContentSize`

    >> block_content_size: UIntGauge,

The total size in bytes of the sum of the transaction content in the block tip.

### `lastBlockEpoch`

    >> block_epoch: UIntGauge,

The epoch of the block date defined in the block tip header.

### `lastBlockSlot`

    >> block_slot: UIntGauge,

The slot of the block date defined in the block tip header.

### `lastBlockHeight`

    >> block_chain_length: UIntGauge,

Length of the blockchain.

### `lastBlockDate`

    >> block_time: UIntGauge,

Timestamp in seconds of the block date.

## Unused metrics

### `lastReceivedBlockTime`

    >> slot_start_time: UIntGauge,

This metric is never updated.

## Unclear metrics

### `lastBlockHashPiece`

    >> block_hash: Vec<UIntGauge>,

A vector of gauges that does something to with the block hash. Metric is updated when `http_response` is called.
