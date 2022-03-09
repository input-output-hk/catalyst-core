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
use evm::{
    backend::{Apply, ApplyBackend, Backend, Basic},
    executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata},
    Context, ExitError, ExitFatal, ExitReason, ExitRevert,
};
use primitive_types::{H160, H256, U256};

use thiserror::Error;

use crate::{
    precompiles::Precompiles,
    state::{AccountTrie, ByteCode, Error as StateError, Key, LogsState},
};

/// Export EVM types
pub use evm::backend::Log;

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

/// A block's base fee per gas.
pub type BlockBaseFeePerGas = U256;

/// A block's coinbase
pub type BlockCoinBase = H160;

/// Gas quantity integer for EVM operations.
pub type Gas = U256;

/// Gas price integer for EVM operations.
pub type GasPrice = U256;

/// Gas limit for EVM operations.
pub type GasLimit = U256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// EVM Configuration parameters needed for execution.
pub enum Config {
    /// Configuration for the `Frontier` fork.
    Frontier = 0,
    /// Configuration for the `Istanbul` fork.
    Istanbul = 1,
    /// Configuration for the `Berlin` fork.
    Berlin = 2,
    /// Configuration for the `London` fork.
    London = 3,
}

impl From<Config> for evm::Config {
    fn from(other: Config) -> Self {
        match other {
            Config::Frontier => Self::frontier(),
            Config::Istanbul => Self::istanbul(),
            Config::Berlin => Self::berlin(),
            Config::London => Self::london(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::Berlin
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
/// EVM Environment parameters needed for execution.
pub struct Environment {
    pub gas_price: GasPrice,
    pub chain_id: ChainId,
    pub block_hashes: BlockHashes,
    pub block_number: BlockNumber,
    pub block_coinbase: BlockCoinBase,
    pub block_timestamp: BlockTimestamp,
    pub block_difficulty: BlockDifficulty,
    pub block_gas_limit: BlockGasLimit,
    pub block_base_fee_per_gas: BlockBaseFeePerGas,
}

/// Integer of the value sent with an EVM transaction.
pub type Value = U256;

/// The context of the EVM runtime
pub type RuntimeContext = Context;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("transaction error: machine returned a normal EVM error")]
    TransactionError(ExitError),
    #[error("transaction fatal error: machine encountered an error that is not supposed to be normal EVM errors, such as requiring too much memory to execute")]
    TransactionFatalError(ExitFatal),
    #[error("transaction has been reverted: machine encountered an explict revert")]
    TransactionRevertError(ExitRevert),
    #[error("state error: {0}")]
    StateError(StateError),
}

pub trait EvmState {
    fn environment(&self) -> &Environment;

    fn accounts(&self) -> &AccountTrie;

    fn logs(&self) -> &LogsState;

    fn update_accounts(&mut self, state: AccountTrie);

    fn update_logs(&mut self, logs: LogsState);
}

fn precompiles(config: Config) -> Precompiles {
    match config {
        Config::Istanbul => Precompiles::new_istanbul(),
        Config::Berlin => Precompiles::new_berlin(),
        // TODO: change it to new_london() after it will be implemented
        Config::London => Precompiles::new_berlin(),
        config => unimplemented!("EVM precompiles for the {:?} config", config),
    }
}

pub struct VirtualMachine<'a, T> {
    state: &'a mut T,
    origin: H160,
}

impl<'a, T> VirtualMachine<'a, T> {
    pub fn new(state: &'a mut T) -> Self {
        Self {
            state,
            origin: Default::default(),
        }
    }
}

/// Top-level abstraction for the EVM with the
/// necessary types used to get the runtime going.
impl<'a, State: EvmState> VirtualMachine<'a, State> {
    fn execute_transaction<F, T>(
        &mut self,
        config: Config,
        caller: Address,
        gas_limit: u64,
        delete_empty: bool,
        f: F,
    ) -> Result<T, Error>
    where
        for<'config> F: FnOnce(
            &mut StackExecutor<'config, '_, MemoryStackState<'_, 'config, Self>, Precompiles>,
            u64,
        ) -> (ExitReason, T),
    {
        self.origin = caller;
        let precompiles = precompiles(config);
        let config = &(config.into());
        let metadata = StackSubstateMetadata::new(gas_limit, config);
        let memory_stack_state = MemoryStackState::new(metadata, self);
        let mut executor =
            StackExecutor::new_with_precompiles(memory_stack_state, config, &precompiles);

        let (exit_reason, val) = f(&mut executor, gas_limit);
        match exit_reason {
            ExitReason::Succeed(_) => {
                // calculate the gas fees given the
                // gas price in the environment
                let gas_fees = executor.fee(self.gas_price());
                // apply changes to the state, this consumes the executor
                let state = executor.into_state();
                // Next, we consume the stack state and extract the values and logs
                // used to modify the accounts trie in the VirtualMachine.
                let (values, logs) = state.deconstruct();

                self.apply(values, logs, delete_empty);

                // pay gas fees
                let new_accounts =
                    self.state
                        .accounts()
                        .clone()
                        .modify_account(caller, |mut account| {
                            account.balance = account.balance.checked_sub(gas_fees)?;
                            Some(account)
                        });

                self.state.update_accounts(new_accounts);

                // exit_reason
                Ok(val)
            }
            ExitReason::Revert(err) => Err(Error::TransactionRevertError(err)),
            ExitReason::Error(err) => Err(Error::TransactionError(err)),
            ExitReason::Fatal(err) => Err(Error::TransactionFatalError(err)),
        }
    }

    /// Execute a CREATE transaction
    #[allow(clippy::too_many_arguments)]
    pub fn transact_create(
        &mut self,
        config: Config,
        caller: Address,
        value: Value,
        init_code: ByteCode,
        gas_limit: u64,
        access_list: Vec<(Address, Vec<Key>)>,
        delete_empty: bool,
    ) -> Result<(), Error> {
        self.execute_transaction(
            config,
            caller,
            gas_limit,
            delete_empty,
            |executor, gas_limit| {
                (
                    executor.transact_create(
                        caller,
                        value,
                        init_code.to_vec(),
                        gas_limit,
                        access_list.clone(),
                    ),
                    (),
                )
            },
        )
    }

    /// Execute a CREATE2 transaction
    #[allow(clippy::too_many_arguments)]
    pub fn transact_create2(
        &mut self,
        config: Config,
        caller: Address,
        value: Value,
        init_code: ByteCode,
        salt: H256,
        gas_limit: u64,
        access_list: Vec<(Address, Vec<Key>)>,
        delete_empty: bool,
    ) -> Result<(), Error> {
        self.execute_transaction(
            config,
            caller,
            gas_limit,
            delete_empty,
            |executor, gas_limit| {
                (
                    executor.transact_create2(
                        caller,
                        value,
                        init_code.to_vec(),
                        salt,
                        gas_limit,
                        access_list.clone(),
                    ),
                    (),
                )
            },
        )
    }

    /// Execute a CALL transaction
    #[allow(clippy::too_many_arguments)]
    pub fn transact_call(
        &mut self,
        config: Config,
        caller: Address,
        address: Address,
        value: Value,
        data: ByteCode,
        gas_limit: u64,
        access_list: Vec<(Address, Vec<Key>)>,
        delete_empty: bool,
    ) -> Result<ByteCode, Error> {
        self.execute_transaction(
            config,
            caller,
            gas_limit,
            delete_empty,
            |executor, gas_limit| {
                executor.transact_call(
                    caller,
                    address,
                    value,
                    data.to_vec(),
                    gas_limit,
                    access_list.clone(),
                )
            },
        )
    }
}

impl<'a, State: EvmState> Backend for VirtualMachine<'a, State> {
    fn gas_price(&self) -> U256 {
        self.state.environment().gas_price
    }
    fn origin(&self) -> H160 {
        self.origin
    }
    fn block_hash(&self, number: U256) -> H256 {
        if number >= self.state.environment().block_number
            || self.state.environment().block_number - number - U256::one()
                >= U256::from(self.state.environment().block_hashes.len())
        {
            H256::default()
        } else {
            let index = (self.state.environment().block_number - number - U256::one()).as_usize();
            self.state.environment().block_hashes[index]
        }
    }
    fn block_number(&self) -> U256 {
        self.state.environment().block_number
    }
    fn block_coinbase(&self) -> H160 {
        self.state.environment().block_coinbase
    }
    fn block_timestamp(&self) -> U256 {
        self.state.environment().block_timestamp
    }
    fn block_difficulty(&self) -> U256 {
        self.state.environment().block_difficulty
    }
    fn block_gas_limit(&self) -> U256 {
        self.state.environment().block_gas_limit
    }
    fn block_base_fee_per_gas(&self) -> U256 {
        self.state.environment().block_base_fee_per_gas
    }
    fn chain_id(&self) -> U256 {
        self.state.environment().chain_id
    }
    fn exists(&self, address: H160) -> bool {
        self.state.accounts().contains(&address)
    }
    fn basic(&self, address: H160) -> Basic {
        self.state
            .accounts()
            .get(&address)
            .map(|a| Basic {
                balance: a.balance.into(),
                nonce: a.nonce,
            })
            .unwrap_or_default()
    }
    fn code(&self, address: H160) -> Vec<u8> {
        self.state
            .accounts()
            .get(&address)
            .map(|val| val.code.to_vec())
            .unwrap_or_default()
    }
    fn storage(&self, address: H160, index: H256) -> H256 {
        self.state
            .accounts()
            .get(&address)
            .map(|val| val.storage.get(&index).cloned().unwrap_or_default())
            .unwrap_or_default()
    }
    fn original_storage(&self, address: H160, index: H256) -> Option<H256> {
        Some(self.storage(address, index))
    }
}

impl<'a, State: EvmState> ApplyBackend for VirtualMachine<'a, State> {
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
                    storage,
                    reset_storage,
                } => {
                    // get the account if stored, else use Default::default().
                    // Then, modify the account balance, nonce, and code.
                    // If reset_storage is set, the account's balance is
                    // set to be Default::default().
                    let new_accounts =
                        self.state
                            .accounts()
                            .clone()
                            .modify_account(address, |mut account| {
                                account.balance = balance.try_into().unwrap();
                                account.nonce = nonce;
                                if let Some(code) = code {
                                    account.code = code
                                };
                                if reset_storage {
                                    account.storage = Default::default();
                                }

                                // cleanup storage from zero values
                                // ref: https://github.com/rust-blockchain/evm/blob/8b1875c83105f47b74d3d7be7302f942e92eb374/src/backend/memory.rs#L185
                                account.storage = account
                                    .storage
                                    .iter()
                                    .filter(|(_, v)| v != &&Default::default())
                                    .map(|(k, v)| (*k, *v))
                                    .collect();

                                // iterate over the apply_storage keys and values
                                // and put them into the account.
                                for (index, value) in storage {
                                    account.storage = if value == Default::default() {
                                        // value is full of zeroes, remove it
                                        account.storage.remove(&index)
                                    } else {
                                        account.storage.put(index, value)
                                    }
                                }

                                if delete_empty && account.is_empty() {
                                    None
                                } else {
                                    Some(account)
                                }
                            });

                    self.state.update_accounts(new_accounts);
                }
                Apply::Delete { address } => {
                    let new_accounts = self.state.accounts().clone().remove(&address);
                    self.state.update_accounts(new_accounts);
                }
            }
        }

        // save the logs
        let block_hash = self.block_hash(self.block_number());
        let mut new_logs = self.state.logs().clone();
        new_logs.put(block_hash, logs.into_iter().collect());
        self.state.update_logs(new_logs);
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod test {
    use super::*;

    struct TestEvmState {
        environment: Environment,
        accounts: AccountTrie,
        logs: LogsState,
    }

    impl EvmState for TestEvmState {
        fn environment(&self) -> &Environment {
            &self.environment
        }

        fn accounts(&self) -> &AccountTrie {
            &self.accounts
        }

        fn logs(&self) -> &LogsState {
            &self.logs
        }

        fn update_accounts(&mut self, accounts: AccountTrie) {
            self.accounts = accounts;
        }

        fn update_logs(&mut self, logs: LogsState) {
            self.logs = logs;
        }
    }

    impl quickcheck::Arbitrary for Config {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            match u8::arbitrary(g) % 4 {
                0 => Config::Frontier,
                1 => Config::Istanbul,
                2 => Config::Berlin,
                3 => Config::London,
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn code_to_execute_evm_runtime_with_defaults_and_no_code_no_data() {
        use evm::{Capture, ExitReason, ExitSucceed, Runtime};
        use std::rc::Rc;

        let config = Config::Istanbul;
        let environment = Environment {
            gas_price: Default::default(),
            chain_id: Default::default(),
            block_hashes: Default::default(),
            block_number: Default::default(),
            block_coinbase: Default::default(),
            block_timestamp: Default::default(),
            block_difficulty: Default::default(),
            block_gas_limit: Default::default(),
            block_base_fee_per_gas: Default::default(),
        };

        let evm_config = config.into();

        let mut evm_state = TestEvmState {
            environment,
            accounts: Default::default(),
            logs: Default::default(),
        };

        let vm = VirtualMachine::new(&mut evm_state);

        let gas_limit = u64::max_value();

        let metadata = StackSubstateMetadata::new(gas_limit, &evm_config);
        let memory_stack_state = MemoryStackState::new(metadata, &vm);
        let precompiles = precompiles(config);
        let mut executor =
            StackExecutor::new_with_precompiles(memory_stack_state, &evm_config, &precompiles);

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
        let mut runtime = Runtime::new(code, data, context, &evm_config);

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
