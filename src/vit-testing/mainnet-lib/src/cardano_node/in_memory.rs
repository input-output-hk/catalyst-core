use crate::cardano_node::Block0;
use crate::cardano_node::Settings;
use crate::Ledger;
use cardano_serialization_lib::{Block, Transaction};
use futures_util::FutureExt;
use jortestkit::process::sleep;
use pharos::{Observable, Observe, ObserveConfig, PharErr, SharedPharos};
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard};
use std::time::Duration;
use tokio::task::JoinHandle;

/// In memory cardano node implementation. Basic design contains read write lock over ledger struct
/// as well as background task for updating ledger
pub struct InMemoryNode {
    block_notifier: SharedPharos<Block>,
    ledger: Arc<RwLock<Ledger>>,
    leadership_process: JoinHandle<()>,
}

impl Observable<Block> for InMemoryNode {
    type Error = PharErr;

    fn observe(&mut self, options: ObserveConfig<Block>) -> Observe<'_, Block, Self::Error> {
        self.block_notifier.observe_shared(options).boxed()
    }
}

impl InMemoryNode {
    /// Retrieves settings carved in block0
    ///
    /// # Errors
    ///
    /// On accessing shard ledger state
    pub fn settings(&self) -> Result<Settings, PoisonError<RwLockReadGuard<'_, Ledger>>> {
        Ok(self.ledger.read()?.settings())
    }

    /// Adds new transaction to mempool
    ///
    /// # Panics
    ///
    /// On accessing shard ledger state
    pub fn push_transaction(&mut self, transaction: Transaction) {
        self.ledger.write().unwrap().push_transaction(transaction);
    }

    /// Adds collection of new transactions to mempool
    ///
    /// # Panics
    ///
    /// On accessing shard ledger state
    pub fn push_transactions(&mut self, transaction: Vec<Transaction>) {
        self.ledger.write().unwrap().push_transactions(transaction);
    }

    /// Waits for all transactions from mempool to be included in block
    ///
    /// # Panics
    ///
    /// On accessing shard ledger state
    pub fn wait_for_txs_to_be_in_block(&self) {
        let settings = self.ledger.read().unwrap().settings();

        while !self.ledger.read().unwrap().mempool().is_empty() {
            sleep(settings.slot_duration.into());
        }
    }

    /// Starts node process
    ///
    /// # Panics
    ///
    /// On accessing ledger state
    #[must_use]
    pub fn start(block0: Block0) -> Self {
        Self::start_from_ledger(Ledger::new(block0))
    }

    /// Starts node process from ledger
    ///
    /// # Panics
    ///
    /// On accessing ledger state
    #[must_use]
    pub fn start_from_ledger(ledger: Ledger) -> Self {
        let slot_duration = ledger.settings().slot_duration;
        let shared_ledger: Arc<RwLock<Ledger>> = Arc::new(RwLock::new(ledger));
        let shared_block_notifier = SharedPharos::default();

        let ledger = shared_ledger.clone();
        let block_notifier = shared_block_notifier.clone();
        let handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(u64::from(slot_duration))).await;
                let block = ledger
                    .write()
                    .unwrap()
                    .mint_block()
                    .expect("leadership event failed");
                block_notifier.notify(block).await.unwrap();
            }
        });

        Self {
            ledger: shared_ledger,
            block_notifier: shared_block_notifier,
            leadership_process: handle,
        }
    }
}

impl Drop for InMemoryNode {
    fn drop(&mut self) {
        self.leadership_process.abort();
    }
}

#[cfg(test)]
mod tests {
    use crate::{Block0, CardanoWallet, InMemoryNode};
    use cardano_serialization_lib::BigNum;
    use futures_util::StreamExt;
    use pharos::{Channel, Observable};

    #[tokio::test]
    pub async fn observer_test() {
        let mut node = InMemoryNode::start(Block0::default());
        let cardano_wallet = CardanoWallet::new(1_000);
        let mut observer = node.observe(Channel::Bounded(1).into()).await.unwrap();

        node.push_transaction(cardano_wallet.generate_direct_voting_registration(0));

        let block = observer.next().await.unwrap();

        assert_eq!(block.header().header_body().block_number(), 1);
        assert_eq!(
            block.header().header_body().slot_bignum(),
            BigNum::from(1u32)
        );
        assert_eq!(block.transaction_bodies().len(), 1);
    }
}
