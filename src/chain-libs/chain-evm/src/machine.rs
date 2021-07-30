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
    backend::{ApplyBackend, Backend, Basic, MemoryVicinity},
    executor::StackState,
    Config, Context,
};
use primitive_types::{H160, H256, U256};

use crate::state::AccountTrie;

pub type Environment = MemoryVicinity;

/// The context of the EVM runtime
pub type RuntimeContext = Context;

/// Top-level abstraction for the EVM with the
/// necessary types used to get the runtime going.
pub struct VirtualMachine {
    _context: RuntimeContext,
    config: MachineConfig,
    state: AccountTrie,
}

/// EVM configuration
pub struct MachineConfig {
    _evm_config: Config,
    environment: Environment,
}

impl Backend for VirtualMachine {
    fn gas_price(&self) -> U256 {
        self.config.environment.gas_price
    }
    fn origin(&self) -> H160 {
        self.config.environment.origin
    }
    fn block_hash(&self, number: U256) -> H256 {
        if number >= self.config.environment.block_number
            || self.config.environment.block_number - number - U256::one()
                >= U256::from(self.config.environment.block_hashes.len())
        {
            H256::default()
        } else {
            let index = (self.config.environment.block_number - number - U256::one()).as_usize();
            self.config.environment.block_hashes[index]
        }
    }
    fn block_number(&self) -> U256 {
        self.config.environment.block_number
    }
    fn block_coinbase(&self) -> H160 {
        self.config.environment.block_coinbase
    }
    fn block_timestamp(&self) -> U256 {
        self.config.environment.block_timestamp
    }
    fn block_difficulty(&self) -> U256 {
        self.config.environment.block_difficulty
    }
    fn block_gas_limit(&self) -> U256 {
        self.config.environment.block_gas_limit
    }
    fn chain_id(&self) -> U256 {
        self.config.environment.chain_id
    }
    fn exists(&self, address: H160) -> bool {
        self.state.contains(&address)
    }
    fn basic(&self, address: H160) -> evm::backend::Basic {
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
            .map(|val| val.code.clone())
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
    fn apply<A, I, L>(&mut self, _values: A, _logs: L, _delete_empty: bool)
    where
        A: IntoIterator<Item = evm::backend::Apply<I>>,
        I: IntoIterator<Item = (H256, H256)>,
        L: IntoIterator<Item = evm::backend::Log>,
    {
        todo!("Add code to apply logs and values in the machine state");
    }
}

impl<'config> StackState<'config> for VirtualMachine {
    fn metadata(&self) -> &evm::executor::StackSubstateMetadata<'config> {
        todo!();
    }
    fn metadata_mut(&mut self) -> &mut evm::executor::StackSubstateMetadata<'config> {
        todo!();
    }
    fn enter(&mut self, _gas_limit: u64, _is_static: bool) {
        todo!();
    }
    fn exit_commit(&mut self) -> Result<(), evm::ExitError> {
        todo!();
    }
    fn exit_revert(&mut self) -> Result<(), evm::ExitError> {
        todo!();
    }
    fn exit_discard(&mut self) -> Result<(), evm::ExitError> {
        todo!();
    }
    fn is_empty(&self, _address: H160) -> bool {
        todo!();
    }
    fn deleted(&self, _address: H160) -> bool {
        todo!();
    }
    fn inc_nonce(&mut self, _address: H160) {
        todo!();
    }
    fn set_storage(&mut self, _address: H160, _key: H256, _value: H256) {
        todo!();
    }
    fn reset_storage(&mut self, _address: H160) {
        todo!();
    }
    fn log(&mut self, _address: H160, _topics: Vec<H256>, _data: Vec<u8>) {
        todo!();
    }
    fn set_deleted(&mut self, _address: H160) {
        todo!();
    }
    fn set_code(&mut self, _address: H160, _code: Vec<u8>) {
        todo!();
    }
    fn transfer(&mut self, _transfer: evm::Transfer) -> Result<(), evm::ExitError> {
        todo!();
    }
    fn reset_balance(&mut self, _address: H160) {
        todo!();
    }
    fn touch(&mut self, _address: H160) {
        todo!();
    }
}

pub fn code_to_execute_evm_runtime() -> Result<(), String> {
    todo!("put together the puzzle of types needed to run evm code");
}
