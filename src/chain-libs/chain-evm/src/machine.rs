//! # The Virtual Machine
//!
//! Abstractions for the EVM runtime.
//!
//! This module contains types that abstract away implementation details from the `evm` crate,
//! specially those involving node and stack configurations, and runtime execution.
//!
//! ## Handler <- EVM Context Handler
//! ## StackState<'config>
//!

use std::rc::Rc;

use evm::{
    backend::{Apply, ApplyBackend, Backend, Basic, Log, MemoryVicinity},
    executor::{MemoryStackState, StackExecutor, StackSubstateMetadata},
    Config, Context, Runtime,
};
use primitive_types::{H160, H256, U256};

use crate::state::AccountTrie;

/// Environment values for the machine backend.
pub type Environment = MemoryVicinity;

/// The context of the EVM runtime
pub type RuntimeContext = Context;

/// Top-level abstraction for the EVM with the
/// necessary types used to get the runtime going.
pub struct VirtualMachine {
    environment: Environment,
    state: AccountTrie,
    logs: Vec<Log>,
}

impl VirtualMachine {
    /// Creates a new `VirtualMachine` given environment values and account storage.
    pub fn new(environment: Environment, state: AccountTrie) -> Self {
        Self {
            environment,
            state,
            logs: Default::default(),
        }
    }

    #[allow(dead_code)]
    /// Returns an initialized instance of `evm::executor::StackExecutor`.
    fn executor<'backend, 'config>(
        &'backend self,
        gas_limit: u64,
        config: &'config Config,
    ) -> StackExecutor<'config, MemoryStackState<'backend, 'config, VirtualMachine>> {
        let metadata = StackSubstateMetadata::new(gas_limit, config);
        let memory_stack_state = MemoryStackState::new(metadata, self);
        StackExecutor::new(memory_stack_state, config)
    }

    #[allow(dead_code)]
    /// Returns an initialized instance of `evm::Runtime`.
    fn runtime<'config>(
        &self,
        code: Rc<Vec<u8>>,
        data: Rc<Vec<u8>>,
        context: RuntimeContext,
        config: &'config Config,
    ) -> Runtime<'config> {
        Runtime::new(code, data, context, config)
    }
}

impl Backend for VirtualMachine {
    fn gas_price(&self) -> U256 {
        self.environment.gas_price
    }
    fn origin(&self) -> H160 {
        self.environment.origin
    }
    fn block_hash(&self, number: U256) -> H256 {
        if number >= self.environment.block_number
            || self.environment.block_number - number - U256::one()
                >= U256::from(self.environment.block_hashes.len())
        {
            H256::default()
        } else {
            let index = (self.environment.block_number - number - U256::one()).as_usize();
            self.environment.block_hashes[index]
        }
    }
    fn block_number(&self) -> U256 {
        self.environment.block_number
    }
    fn block_coinbase(&self) -> H160 {
        self.environment.block_coinbase
    }
    fn block_timestamp(&self) -> U256 {
        self.environment.block_timestamp
    }
    fn block_difficulty(&self) -> U256 {
        self.environment.block_difficulty
    }
    fn block_gas_limit(&self) -> U256 {
        self.environment.block_gas_limit
    }
    fn chain_id(&self) -> U256 {
        self.environment.chain_id
    }
    fn exists(&self, address: H160) -> bool {
        self.state.contains(&address)
    }
    fn basic(&self, address: H160) -> Basic {
        self.state
            .get(&address)
            .map(|a| Basic {
                balance: a.balance,
                nonce: a.nonce,
            })
            .unwrap_or_default()
    }
    fn code(&self, address: H160) -> Vec<u8> {
        self.state
            .get(&address)
            .map(|val| val.code.to_vec())
            .unwrap_or_default()
    }
    fn storage(&self, address: H160, index: H256) -> H256 {
        self.state
            .get(&address)
            .map(|val| val.storage.get(&index).cloned().unwrap_or_default())
            .unwrap_or_default()
    }
    fn original_storage(&self, address: H160, index: H256) -> Option<H256> {
        Some(self.storage(address, index))
    }
}

impl ApplyBackend for VirtualMachine {
    fn apply<A, I, L>(&mut self, values: A, logs: L, delete_empty: bool)
    where
        A: IntoIterator<Item = Apply<I>>,
        I: IntoIterator<Item = (H256, H256)>,
        L: IntoIterator<Item = Log>,
    {
        for apply in values {
            match apply {
                Apply::Modify {
                    address,
                    basic: Basic { balance, nonce },
                    code,
                    storage: apply_storage,
                    reset_storage,
                } => {
                    // get the account if stored, else use Default::default().
                    // Then, modify the account balance, nonce, and code.
                    // If reset_storage is set, the account's balance is
                    // set to be Default::default().
                    let mut account =
                        self.state
                            .modify_account(&address, balance, nonce, code, reset_storage);

                    // iterate over the apply_storage keys and values
                    // and put them into the account.
                    for (index, value) in apply_storage {
                        account.storage = if value == crate::state::Value::default() {
                            // value is full of zeroes, remove it
                            account.storage.clone().remove(&index)
                        } else {
                            account.storage.clone().put(index, value)
                        }
                    }

                    self.state = if delete_empty && account.is_empty() {
                        self.state.clone().remove(&address)
                    } else {
                        self.state.clone().put(address, account)
                    }
                }
                Apply::Delete { address } => {
                    self.state = self.state.clone().remove(&address);
                }
            }
        }

        // save the logs
        for log in logs {
            self.logs.push(log);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use evm::{Capture, ExitReason, ExitSucceed};

    use super::*;

    #[test]
    fn code_to_execute_evm_runtime_with_defaults_and_no_code_no_data() {
        let environment = Environment {
            gas_price: Default::default(),
            origin: Default::default(),
            chain_id: Default::default(),
            block_hashes: Default::default(),
            block_number: Default::default(),
            block_coinbase: Default::default(),
            block_timestamp: Default::default(),
            block_difficulty: Default::default(),
            block_gas_limit: Default::default(),
        };
        let state = AccountTrie::default();

        let vm = VirtualMachine::new(environment, state);

        let gas_limit = u64::max_value();
        let config = Config::istanbul();
        let mut executor = vm.executor(gas_limit, &config);

        // Byte-encoded smart contract code
        let code = Rc::new(Vec::new());
        // Byte-encoded input data used for smart contract code
        let data = Rc::new(Vec::new());
        // EVM Context
        let context = RuntimeContext {
            address: Default::default(),
            caller: Default::default(),
            apparent_value: Default::default(),
        };
        // Handle for the EVM runtime
        let mut runtime = vm.runtime(code, data, context, &config);

        let exit_reason = runtime.run(&mut executor);

        if let Capture::Exit(ExitReason::Succeed(ExitSucceed::Stopped)) = exit_reason {
            // We consume the executor to extract the stack state after executing
            // the code.
            let state = executor.into_state();
            // Next, we consume the stack state and extract the values and logs
            // used to modify the accounts trie in the VirtualMachine.
            let (values, logs) = state.deconstruct();

            // We assert that there are no values or logs from the code execution.
            assert_eq!(0, values.into_iter().count());
            assert_eq!(0, logs.into_iter().count());
            // // Here is where we would apply the changes in the backend
            // vm.apply(values, logs, true);
        } else {
            panic!("unexpected evm result");
        }
    }
}
