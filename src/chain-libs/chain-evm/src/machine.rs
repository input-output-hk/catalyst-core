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
    backend::{Apply, ApplyBackend, Backend, Basic},
    executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata},
    Context, Runtime,
};
use primitive_types::{H160, H256, U256};

use crate::{
    precompiles::Precompiles,
    state::{AccountTrie, ByteCode, Key, LogsState},
};

/// Export EVM types
pub use evm::{
    backend::{Log, MemoryVicinity as Environment},
    Config, ExitReason,
};

/// An address of an EVM account.
pub type Address = H160;

/// A block's chain ID.
pub type ChainId = U256;

/// A block hash.
pub type BlockHash = H256;

/// A block hash.
pub type BlockHashes = Vec<BlockHash>;

/// A block's number.
pub type BlockNumber = U256;

/// A block's timestamp.
pub type BlockTimestamp = U256;

/// A block's difficulty.
pub type BlockDifficulty = U256;

/// A block's gas limit.
pub type BlockGasLimit = U256;

/// A block's origin
pub type Origin = H160;

/// A block's coinbase
pub type BlockCoinBase = H160;

/// Gas quantity integer for EVM operations.
pub type Gas = U256;

/// Gas price integer for EVM operations.
pub type GasPrice = U256;

/// Gas limit for EVM operations.
pub type GasLimit = U256;

/// Integer of the value sent with an EVM transaction.
pub type Value = U256;

/// The context of the EVM runtime
pub type RuntimeContext = Context;

/// Top-level abstraction for the EVM with the
/// necessary types used to get the runtime going.
pub struct VirtualMachine<'runtime> {
    /// EVM Block Configuration.
    config: &'runtime Config,
    environment: &'runtime Environment,
    precompiles: Precompiles,
    state: AccountTrie,
    logs: LogsState,
}

/// Ethereum Hard-Fork variants
pub enum HardFork {
    Istanbul,
    Berlin,
}

fn precompiles(fork: HardFork) -> Precompiles {
    match fork {
        HardFork::Istanbul => Precompiles::new_istanbul(),
        HardFork::Berlin => Precompiles::new_berlin(),
    }
}

impl<'runtime> VirtualMachine<'runtime> {
    /// Creates a new `VirtualMachine` given configuration parameters.
    pub fn new(config: &'runtime Config, environment: &'runtime Environment) -> Self {
        Self::new_with_state(config, environment, Default::default())
    }

    /// Creates a new `VirtualMachine` given configuration params and a given account storage.
    pub fn new_with_state(
        config: &'runtime Config,
        environment: &'runtime Environment,
        state: AccountTrie,
    ) -> Self {
        Self {
            config,
            environment,
            precompiles: precompiles(HardFork::Berlin),
            state,
            logs: Default::default(),
        }
    }

    /// Returns an initialized instance of `evm::Runtime`.
    pub fn runtime(
        &self,
        code: Rc<Vec<u8>>,
        data: Rc<Vec<u8>>,
        context: RuntimeContext,
    ) -> Runtime<'_> {
        Runtime::new(code, data, context, self.config)
    }

    /// Execute a CREATE transaction
    #[allow(clippy::boxed_local)]
    pub fn transact_create(
        &mut self,
        caller: Address,
        value: Value,
        init_code: ByteCode,
        gas_limit: u64,
        access_list: Vec<(Address, Vec<Key>)>,
        delete_empty: bool,
    ) -> Option<(&AccountTrie, &LogsState)> {
        {
            let metadata = StackSubstateMetadata::new(gas_limit, self.config);
            let memory_stack_state = MemoryStackState::new(metadata, self);
            let mut executor = StackExecutor::new_with_precompiles(
                memory_stack_state,
                self.config,
                &self.precompiles,
            );

            let exit_reason =
                executor.transact_create(caller, value, init_code.to_vec(), gas_limit, access_list);
            match exit_reason {
                ExitReason::Succeed(_succeded) => {
                    // apply and return state
                    // apply changes to the state, this consumes the executor
                    let state = executor.into_state();
                    // Next, we consume the stack state and extract the values and logs
                    // used to modify the accounts trie in the VirtualMachine.
                    let (values, logs) = state.deconstruct();

                    self.apply(values, logs, delete_empty);
                    //_exit_reason
                    Some((&self.state, &self.logs))
                }
                _ => None,
            }
        }
    }

    /// Execute a CREATE2 transaction
    #[allow(clippy::boxed_local)]
    #[allow(clippy::too_many_arguments)]
    pub fn transact_create2(
        &mut self,
        caller: Address,
        value: Value,
        init_code: ByteCode,
        salt: H256,
        gas_limit: u64,
        access_list: Vec<(Address, Vec<Key>)>,
        delete_empty: bool,
    ) -> Option<(&AccountTrie, &LogsState)> {
        {
            let metadata = StackSubstateMetadata::new(gas_limit, self.config);
            let memory_stack_state = MemoryStackState::new(metadata, self);
            let mut executor = StackExecutor::new_with_precompiles(
                memory_stack_state,
                self.config,
                &self.precompiles,
            );
            let exit_reason = executor.transact_create2(
                caller,
                value,
                init_code.to_vec(),
                salt,
                gas_limit,
                access_list,
            );
            match exit_reason {
                ExitReason::Succeed(_succeded) => {
                    // apply and return state
                    // apply changes to the state, this consumes the executor
                    let state = executor.into_state();
                    // Next, we consume the stack state and extract the values and logs
                    // used to modify the accounts trie in the VirtualMachine.
                    let (values, logs) = state.deconstruct();

                    self.apply(values, logs, delete_empty);
                    //_exit_reason
                    Some((&self.state, &self.logs))
                }
                _ => None,
            }
        }
    }

    /// Execute a CALL transaction
    #[allow(clippy::boxed_local)]
    #[allow(clippy::too_many_arguments)]
    pub fn transact_call(
        &mut self,
        caller: Address,
        address: Address,
        value: Value,
        data: ByteCode,
        gas_limit: u64,
        access_list: Vec<(Address, Vec<Key>)>,
        delete_empty: bool,
    ) -> Option<(&AccountTrie, &LogsState, ByteCode)> {
        let metadata = StackSubstateMetadata::new(gas_limit, self.config);
        let memory_stack_state = MemoryStackState::new(metadata, self);
        let mut executor =
            StackExecutor::new_with_precompiles(memory_stack_state, self.config, &self.precompiles);
        let (exit_reason, byte_output) = executor.transact_call(
            caller,
            address,
            value,
            data.to_vec(),
            gas_limit,
            access_list,
        );
        match exit_reason {
            ExitReason::Succeed(_succeded) => {
                // apply and return state
                // apply changes to the state, this consumes the executor
                let state = executor.into_state();
                // Next, we consume the stack state and extract the values and logs
                // used to modify the accounts trie in the VirtualMachine.
                let (values, logs) = state.deconstruct();

                self.apply(values, logs, delete_empty);
                //_exit_reason
                Some((&self.state, &self.logs, byte_output.into_boxed_slice()))
            }
            _ => None,
        }
    }
}

impl<'runtime> Backend for VirtualMachine<'runtime> {
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
    fn block_base_fee_per_gas(&self) -> U256 {
        self.environment.block_base_fee_per_gas
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

impl<'runtime> ApplyBackend for VirtualMachine<'runtime> {
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
        let block_hash = self.block_hash(self.block_number());
        self.logs.put(block_hash, logs.into_iter().collect());
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use evm::{Capture, ExitReason, ExitSucceed};

    use super::*;

    #[test]
    fn code_to_execute_evm_runtime_with_defaults_and_no_code_no_data() {
        let config = Config::istanbul();
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
            block_base_fee_per_gas: Default::default(),
        };

        let vm = VirtualMachine::new(&config, &environment);

        let gas_limit = u64::max_value();

        let metadata = StackSubstateMetadata::new(gas_limit, &config);
        let memory_stack_state = MemoryStackState::new(metadata, &vm);
        let precompiles = precompiles(HardFork::Berlin);
        let mut executor =
            StackExecutor::new_with_precompiles(memory_stack_state, &config, &precompiles);

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
        let mut runtime = vm.runtime(code, data, context);

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
