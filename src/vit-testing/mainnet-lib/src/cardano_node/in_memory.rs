use std::sync::{Arc, RwLock};
use cardano_serialization_lib::{Block, Transaction};
use tokio::task::JoinHandle;
use jortestkit::process::sleep;
use crate::Ledger;
use pharos::{Observable, Observe, ObserveConfig, PharErr, SharedPharos};
use futures_util::FutureExt;
use crate::cardano_node::block::Block0;

pub struct InMemoryNode {
    block_notifier: SharedPharos<Block>,
    ledger: Arc<RwLock<Ledger>>,
    handle: JoinHandle<()>
}

impl Observable<Block> for InMemoryNode
{
    type Error = PharErr;

    fn observe( &mut self, options: ObserveConfig<Block>) -> Observe< '_, Block, Self::Error >
    {
        self.block_notifier.observe_shared( options ).boxed()
    }
}

impl InMemoryNode {

    pub fn push_transaction(&mut self, transaction: Transaction) {
        self.ledger.write().unwrap().push_transaction(transaction);
    }

    pub fn push_transactions(&mut self, transaction: Vec<Transaction>) {
        self.ledger.write().unwrap().push_transactions(transaction);
    }

    pub fn wait_for_txs_to_be_in_block(&self) {
        let settings = self.ledger.read().unwrap().settings().clone();

        while !self.ledger.read().unwrap().mempool().is_empty() {
            sleep(settings.slot_duration as u64);
        }
    }

    pub fn start(block0: Block0) -> Self {
        let slot_duration = block0.settings.slot_duration;
        let shared_ledger: Arc<RwLock<Ledger>> = Arc::new(RwLock::new(Ledger::new(block0)));
        let shared_block_notifier = SharedPharos::default();

        let ledger = shared_ledger.clone();
        let block_notifier = shared_block_notifier.clone();

        let handle = tokio::spawn( async move {
            loop {
                sleep(slot_duration.into());
                let block = ledger.write().unwrap().mint_block();
                block_notifier.notify(block).await.unwrap();
            }
        });

        Self {
            ledger: shared_ledger,
            block_notifier: shared_block_notifier,
            handle
        }
    }
}

impl Drop for InMemoryNode {
    fn drop(&mut self) {
        self.handle.abort();
    }
}