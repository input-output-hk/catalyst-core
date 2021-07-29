//! Abstractions for the EVM runtime.
//!
//! This module contains types that abstract away implementation details from the `evm` crate,
//! specially those involving node and stack configurations, and runtime execution.

use evm::{
    backend::{ApplyBackend, Backend},
};
use primitive_types::{H160, H256, U256};


pub struct Machine {
    //
}

impl Backend for Machine {
    fn gas_price(&self) -> U256 {
        todo!();
    }
    fn origin(&self) -> H160 {
        todo!();
    }
    fn block_hash(&self, number: U256) -> H256 {
        todo!();
    }
    fn block_number(&self) -> U256 {
        todo!();
    }
    fn block_coinbase(&self) -> H160 {
        todo!();
    }
    fn block_timestamp(&self) -> U256 {
        todo!();
    }
    fn block_difficulty(&self) -> U256 {
        todo!();
    }
    fn block_gas_limit(&self) -> U256 {
        todo!();
    }
    fn chain_id(&self) -> U256 {
        todo!();
    }
    fn exists(&self, address: H160) -> bool {
        todo!();
    }
    fn basic(&self, address: H160) -> evm::backend::Basic {
        todo!();
    }
    fn code(&self, address: H160) -> Vec<u8> {
        todo!();
    }
    fn storage(&self, address: H160, index: H256) -> H256 {
        todo!();
    }
    fn original_storage(&self, address: H160, index: H256) -> Option<H256> {
        todo!();
    }
}

impl ApplyBackend for Machine {
    fn apply<A, I, L>(&mut self, values: A, logs: L, delete_empty: bool)
    where
        A: IntoIterator<Item = evm::backend::Apply<I>>,
        I: IntoIterator<Item = (H256, H256)>,
        L: IntoIterator<Item = evm::backend::Log>,
    {
        todo!("Add code to apply logs and values in the machine state");
    }
}

pub fn code_to_execute_evm_runtime() -> Result<(), ()> {
    todo!("put together the puzzle of types needed to run evm code");
}
