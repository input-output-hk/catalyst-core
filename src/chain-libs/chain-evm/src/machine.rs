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
    backend::{Apply, ApplyBackend, Backend, Basic, Log, MemoryVicinity},
    executor::StackState,
    Context,
};
use primitive_types::{H160, H256, U256};

use crate::state::{Account, AccountAddress, AccountTrie, Balance, Nonce};

/// Environment values for the machine backend.
pub type Environment = MemoryVicinity;

/// The context of the EVM runtime
pub type RuntimeContext = Context;

/// Top-level abstraction for the EVM with the
/// necessary types used to get the runtime going.
pub struct VirtualMachine {
    _context: RuntimeContext,
    environment: Environment,
    state: AccountTrie,
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

fn modify_account(
    state: &AccountTrie,
    address: &AccountAddress,
    balance: Balance,
    nonce: Nonce,
    reset_storage: bool,
    code: Option<Vec<u8>>,
) -> Account {
    let account = if let Some(acct) = state.get(&address) {
        acct.clone()
    } else {
        Default::default()
    };
    let storage = if reset_storage {
        Default::default()
    } else {
        account.storage
    };
    let code = if let Some(code) = code {
        code
    } else {
        account.code
    };

    Account {
        nonce,
        balance,
        storage,
        code,
    }
}

impl ApplyBackend for VirtualMachine {
    fn apply<A, I, L>(&mut self, values: A, _logs: L, _delete_empty: bool)
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
                    storage: _apply_storage,
                    reset_storage,
                } => {
                    // get the account if stored, else use default
                    let _account =
                        modify_account(&self.state, &address, balance, nonce, reset_storage, code);

                    // WIP:
                    todo!();
                }
                Apply::Delete { address: _ } => {
                    todo!();
                }
            }
        }
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
