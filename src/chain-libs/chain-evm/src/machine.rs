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
use crate::{
    precompiles::Precompiles,
    state::{Account, Balance, ByteCode, Key},
};
use ethereum_types::{H160, H256, U256};
use evm::{
    backend::{Backend, Basic},
    executor::stack::{Accessed, StackExecutor, StackState, StackSubstateMetadata},
    Context, ExitFatal, ExitReason, ExitRevert, Transfer,
};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Export EVM types
pub use evm::backend::Log;
pub use evm::ExitError;

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
}

pub trait EvmState {
    fn environment(&self) -> &Environment;

    fn account(&self, address: &Address) -> Option<Account>;

    fn contains(&self, address: &Address) -> bool;

    fn modify_account<F>(&mut self, address: Address, f: F) -> Result<(), ExitError>
    where
        F: FnOnce(Account) -> Option<Account>;

    fn update_logs(&mut self, block_hash: BlockHash, logs: Vec<Log>);
}

struct VirtualMachineSubstate<'a> {
    metadata: StackSubstateMetadata<'a>,
    logs: Vec<Log>,
    accounts: BTreeMap<H160, Account>,
    deletes: BTreeSet<H160>,
    parent: Option<Box<VirtualMachineSubstate<'a>>>,
}

impl<'a> VirtualMachineSubstate<'a> {
    fn known_account(&self, address: &H160) -> Option<&Account> {
        if let Some(account) = self.accounts.get(address) {
            Some(account)
        } else if let Some(parent) = self.parent.as_ref() {
            parent.known_account(address)
        } else {
            None
        }
    }

    fn deleted(&self, address: &H160) -> bool {
        if self.deletes.contains(address) {
            return true;
        }
        if let Some(parent) = self.parent.as_ref() {
            return parent.deleted(address);
        }
        false
    }

    fn recursive_is_cold<F: Fn(&Accessed) -> bool>(&self, f: &F) -> bool {
        let local_is_accessed = self.metadata.accessed().as_ref().map(f).unwrap_or(false);
        if local_is_accessed {
            false
        } else {
            self.parent
                .as_ref()
                .map(|p| p.recursive_is_cold(f))
                .unwrap_or(true)
        }
    }

    fn account_mut<State: EvmState>(&mut self, address: H160, state: &State) -> &mut Account {
        let account = self
            .known_account(&address)
            .cloned()
            .unwrap_or_else(|| state.account(&address).unwrap_or_default());
        self.accounts.entry(address).or_insert(account)
    }
}

pub struct VirtualMachine<'a, T> {
    state: &'a mut T,
    config: &'a evm::Config,
    origin: H160,
    gas_limit: u64,
    delete_empty: bool,
    substate: VirtualMachineSubstate<'a>,
}

impl<'a, T> VirtualMachine<'a, T> {
    pub fn new(
        state: &'a mut T,
        config: &'a evm::Config,
        origin: H160,
        gas_limit: u64,
        delete_empty: bool,
    ) -> Self {
        Self {
            state,
            config,
            origin,
            gas_limit,
            delete_empty,
            substate: VirtualMachineSubstate {
                metadata: StackSubstateMetadata::new(gas_limit, config),
                logs: Default::default(),
                accounts: Default::default(),
                deletes: Default::default(),
                parent: None,
            },
        }
    }
}

/// Top-level abstraction for the EVM with the
/// necessary types used to get the runtime going.
fn execute_transaction<State: EvmState, F, T>(vm: VirtualMachine<State>, f: F) -> Result<T, Error>
where
    for<'config> F: FnOnce(
        &mut StackExecutor<'config, '_, VirtualMachine<'config, State>, Precompiles>,
    ) -> (ExitReason, T),
{
    let precompiles = Precompiles::new();
    let config = vm.config;
    let gas_price = vm.gas_price();

    // let memory_stack_state = MemoryStackState::new(vm.substate.metadata.clone(), &vm);
    let mut executor = StackExecutor::new_with_precompiles(vm, config, &precompiles);

    let (exit_reason, val) = f(&mut executor);
    match exit_reason {
        ExitReason::Succeed(_) => {
            // calculate the gas fees given the
            // gas price in the environment
            let gas_fees = executor.fee(gas_price);
            // apply changes to the state, this consumes the executor
            let vm = executor.into_state();

            // pay gas fees
            if let Some(mut account) = vm.state.account(&vm.origin) {
                account.balance = account
                    .balance
                    .checked_sub(
                        gas_fees
                            .try_into()
                            .map_err(|_| Error::TransactionError(ExitError::OutOfFund))?,
                    )
                    .ok_or(Error::TransactionError(ExitError::OutOfFund))?;
                vm.state
                    .modify_account(vm.origin, |_| Some(account))
                    .map_err(Error::TransactionError)?;
            }

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
pub fn transact_create<State: EvmState>(
    vm: VirtualMachine<State>,
    value: U256,
    init_code: ByteCode,
    access_list: Vec<(Address, Vec<Key>)>,
) -> Result<ByteCode, Error> {
    let caller = vm.origin;
    let gas_limit = vm.gas_limit;
    execute_transaction(vm, |executor| {
        executor.transact_create(
            caller,
            value,
            init_code.to_vec(),
            gas_limit,
            access_list.clone(),
        )
    })
}

/// Execute a CREATE2 transaction
#[allow(clippy::too_many_arguments)]
pub fn transact_create2<State: EvmState>(
    vm: VirtualMachine<State>,
    value: U256,
    init_code: ByteCode,
    salt: H256,
    access_list: Vec<(Address, Vec<Key>)>,
) -> Result<ByteCode, Error> {
    let caller = vm.origin;
    let gas_limit = vm.gas_limit;
    execute_transaction(vm, |executor| {
        executor.transact_create2(
            caller,
            value,
            init_code.to_vec(),
            salt,
            gas_limit,
            access_list.clone(),
        )
    })
}

/// Execute a CALL transaction
#[allow(clippy::too_many_arguments)]
pub fn transact_call<State: EvmState>(
    vm: VirtualMachine<State>,
    address: Address,
    value: U256,
    data: ByteCode,
    access_list: Vec<(Address, Vec<Key>)>,
) -> Result<ByteCode, Error> {
    let caller = vm.origin;
    let gas_limit = vm.gas_limit;
    execute_transaction(vm, |executor| {
        executor.transact_call(
            caller,
            address,
            value,
            data.to_vec(),
            gas_limit,
            access_list.clone(),
        )
    })
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
        self.substate.known_account(&address).is_some() || self.state.contains(&address)
    }
    fn basic(&self, address: H160) -> Basic {
        self.substate
            .known_account(&address)
            .map(|account| Basic {
                balance: account.balance.into(),
                nonce: account.state.nonce,
            })
            .unwrap_or_else(|| {
                self.state
                    .account(&address)
                    .map(|account| Basic {
                        balance: account.balance.into(),
                        nonce: account.state.nonce,
                    })
                    .unwrap_or_default()
            })
    }
    fn code(&self, address: H160) -> Vec<u8> {
        self.substate
            .known_account(&address)
            .map(|account| account.state.code.clone())
            .unwrap_or_else(|| {
                self.state
                    .account(&address)
                    .map(|account| account.state.code)
                    .unwrap_or_default()
            })
    }
    fn storage(&self, address: H160, index: H256) -> H256 {
        self.substate
            .known_account(&address)
            .map(|account| {
                account
                    .state
                    .storage
                    .get(&index)
                    .cloned()
                    .unwrap_or_default()
            })
            .unwrap_or_else(|| {
                self.state
                    .account(&address)
                    .map(|account| {
                        account
                            .state
                            .storage
                            .get(&index)
                            .cloned()
                            .unwrap_or_default()
                    })
                    .unwrap_or_default()
            })
    }
    fn original_storage(&self, address: H160, index: H256) -> Option<H256> {
        match self.state.account(&address) {
            Some(account) => account.state.storage.get(&index).cloned(),
            None => None,
        }
    }
}

impl<'a, State: EvmState> StackState<'a> for VirtualMachine<'a, State> {
    fn metadata(&self) -> &StackSubstateMetadata<'a> {
        &self.substate.metadata
    }

    fn metadata_mut(&mut self) -> &mut StackSubstateMetadata<'a> {
        &mut self.substate.metadata
    }

    fn enter(&mut self, gas_limit: u64, is_static: bool) {
        let mut entering = VirtualMachineSubstate {
            metadata: self.substate.metadata.spit_child(gas_limit, is_static),
            logs: Default::default(),
            accounts: Default::default(),
            deletes: Default::default(),
            parent: None,
        };
        core::mem::swap(&mut entering, &mut self.substate);

        self.substate.parent = Some(Box::new(entering));
    }

    fn exit_commit(&mut self) -> Result<(), ExitError> {
        let mut exited = *self
            .substate
            .parent
            .take()
            .expect("Cannot commit on root substate");
        core::mem::swap(&mut exited, &mut self.substate);

        self.substate.metadata.swallow_commit(exited.metadata)?;

        self.substate.accounts.append(&mut exited.accounts);
        self.substate.deletes.append(&mut exited.deletes);
        self.substate.logs.append(&mut exited.logs);

        // Apply changes

        // apply accounts
        for (address, account) in self.substate.accounts.iter() {
            let mut account = account.clone();
            // cleanup storage from zero values
            // ref: https://github.com/rust-blockchain/evm/blob/8b1875c83105f47b74d3d7be7302f942e92eb374/src/backend/memory.rs#L185
            account.state.storage = account
                .state
                .storage
                .iter()
                .filter(|(_, v)| v != &&Default::default())
                .map(|(k, v)| (*k, *v))
                .collect();

            self.state.modify_account(*address, |_| {
                if self.delete_empty && account.is_empty() {
                    None
                } else {
                    Some(account)
                }
            })?;
        }

        // delete account
        for address in self.substate.deletes.iter() {
            self.state.modify_account(*address, |_| None)?;
        }

        // save the logs
        let block_hash = self.block_hash(self.block_number());
        self.state
            .update_logs(block_hash, self.substate.logs.clone().into_iter().collect());

        Ok(())
    }

    fn exit_revert(&mut self) -> Result<(), ExitError> {
        let mut exited = *self
            .substate
            .parent
            .take()
            .expect("Cannot discard on root substate");
        core::mem::swap(&mut exited, &mut self.substate);

        self.substate.metadata.swallow_revert(exited.metadata)
    }

    fn exit_discard(&mut self) -> Result<(), ExitError> {
        let mut exited = *self
            .substate
            .parent
            .take()
            .expect("Cannot discard on root substate");
        core::mem::swap(&mut exited, &mut self.substate);

        self.substate.metadata.swallow_discard(exited.metadata)
    }

    fn is_empty(&self, address: H160) -> bool {
        match self.substate.known_account(&address) {
            Some(account) => account.is_empty(),
            None => match self.state.account(&address) {
                Some(account) => account.is_empty(),
                None => true,
            },
        }
    }

    fn deleted(&self, address: H160) -> bool {
        self.substate.deleted(&address)
    }

    fn is_cold(&self, address: H160) -> bool {
        self.substate
            .recursive_is_cold(&|a| a.accessed_addresses.contains(&address))
    }

    fn is_storage_cold(&self, address: H160, key: H256) -> bool {
        self.substate
            .recursive_is_cold(&|a: &Accessed| a.accessed_storage.contains(&(address, key)))
    }

    fn inc_nonce(&mut self, address: H160) {
        self.substate.account_mut(address, self.state).state.nonce += U256::one();
    }

    fn set_storage(&mut self, address: H160, key: H256, value: H256) {
        let account = self.substate.account_mut(address, self.state);
        account.state.storage = account.state.storage.clone().put(key, value);
    }

    fn reset_storage(&mut self, address: H160) {
        self.substate.account_mut(address, self.state).state.storage = Default::default();
    }

    fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) {
        self.substate.logs.push(Log {
            address,
            topics,
            data,
        });
    }

    fn set_deleted(&mut self, address: H160) {
        self.substate.deletes.insert(address);
    }

    fn set_code(&mut self, address: H160, code: Vec<u8>) {
        self.substate.account_mut(address, self.state).state.code = code;
    }

    fn transfer(&mut self, transfer: Transfer) -> Result<(), ExitError> {
        let source = self.substate.account_mut(transfer.source, self.state);

        source.balance = match source.balance.checked_sub(
            transfer
                .value
                .try_into()
                .map_err(|_| ExitError::OutOfFund)?,
        ) {
            Some(res) => res,
            None => return Err(ExitError::OutOfFund),
        };

        let target = self.substate.account_mut(transfer.target, self.state);
        target.balance = match target.balance.checked_add(
            transfer
                .value
                .try_into()
                .map_err(|_| ExitError::Other("Balance overflow".into()))?,
        ) {
            Some(res) => res,
            None => return Err(ExitError::Other("Balance overflow".into())),
        };

        Ok(())
    }

    fn reset_balance(&mut self, address: H160) {
        self.substate.account_mut(address, self.state).balance = Balance::zero()
    }

    fn touch(&mut self, address: H160) {
        self.substate.account_mut(address, self.state);
    }
}

#[cfg(any(test, feature = "property-test-api"))]
pub mod test {
    use super::*;
    use crate::state::{AccountTrie, LogsState};
    #[cfg(test)]
    use evm::executor::stack::MemoryStackState;

    pub struct TestEvmState {
        pub environment: Environment,
        pub accounts: AccountTrie,
        pub logs: LogsState,
    }

    impl EvmState for TestEvmState {
        fn environment(&self) -> &Environment {
            &self.environment
        }

        fn account(&self, address: &Address) -> Option<Account> {
            self.accounts.get(address).cloned()
        }

        fn contains(&self, address: &Address) -> bool {
            self.accounts.contains(address)
        }

        fn modify_account<F>(&mut self, address: Address, f: F) -> Result<(), ExitError>
        where
            F: FnOnce(Account) -> Option<Account>,
        {
            self.accounts = self.accounts.clone().modify_account(address, f);
            Ok(())
        }

        fn update_logs(&mut self, block_hash: H256, logs: Vec<Log>) {
            self.logs.put(block_hash, logs);
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

        let config = Config::Istanbul.into();
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

        let mut evm_state = TestEvmState {
            environment,
            accounts: Default::default(),
            logs: Default::default(),
        };

        let caller = Default::default();
        let gas_limit = u64::max_value();

        let vm = VirtualMachine::new(&mut evm_state, &config, caller, gas_limit, true);

        let metadata = StackSubstateMetadata::new(gas_limit, &config);
        let memory_stack_state = MemoryStackState::new(metadata, &vm);
        let precompiles = Precompiles::new();
        let mut executor =
            StackExecutor::new_with_precompiles(memory_stack_state, &config, &precompiles);

        // Byte-encoded smart contract code
        let code = Rc::new(Vec::new());
        // Byte-encoded input data used for smart contract code
        let data = Rc::new(Vec::new());
        // EVM Context
        let context = RuntimeContext {
            address: Default::default(),
            caller,
            apparent_value: Default::default(),
        };
        // Handle for the EVM runtime
        let mut runtime = Runtime::new(code, data, context, &config);

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
