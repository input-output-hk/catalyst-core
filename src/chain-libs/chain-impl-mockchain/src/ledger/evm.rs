use crate::evm::EvmTransaction;
use crate::ledger::Error;
use chain_evm::{
    machine::{Config, Environment, VirtualMachine},
    state::{AccountTrie, Balance, LogsState},
};

#[derive(Clone, PartialEq, Eq)]
pub struct Ledger {
    pub(crate) accounts: AccountTrie,
    pub(crate) logs: LogsState,
    pub(crate) environment: Environment,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            accounts: Default::default(),
            logs: Default::default(),
            environment: Environment {
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
            },
        }
    }
}

impl Ledger {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn run_transaction(
        &mut self,
        contract: EvmTransaction,
        config: &Config,
    ) -> Result<(), Error> {
        let mut vm = VirtualMachine::new_with_state(
            config,
            &self.environment,
            self.accounts.clone(),
            self.logs.clone(),
        );
        match contract {
            EvmTransaction::Create {
                caller,
                value,
                init_code,
                gas_limit,
                access_list,
            } => {
                //
                let (new_state, new_logs) =
                    vm.transact_create(caller, value, init_code, gas_limit, access_list, true)?;
                // update ledger state
                self.accounts = new_state.clone();
                self.logs = new_logs.clone();
                Ok(())
            }
            EvmTransaction::Create2 {
                caller,
                value,
                init_code,
                salt,
                gas_limit,
                access_list,
            } => {
                let (new_state, new_logs) = vm.transact_create2(
                    caller,
                    value,
                    init_code,
                    salt,
                    gas_limit,
                    access_list,
                    true,
                )?;
                // update ledger state
                self.accounts = new_state.clone();
                self.logs = new_logs.clone();
                Ok(())
            }
            EvmTransaction::Call {
                caller,
                address,
                value,
                data,
                gas_limit,
                access_list,
            } => {
                let (new_state, new_logs, _byte_code_msg) =
                    vm.transact_call(caller, address, value, data, gas_limit, access_list, true)?;
                // update ledger state
                self.accounts = new_state.clone();
                self.logs = new_logs.clone();
                Ok(())
            }
        }
    }
}

impl Ledger {
    pub(crate) fn stats(&self) -> String {
        let Ledger { accounts, .. } = self;
        let mut count = 0;
        let mut total = Balance::zero();
        for (_, account) in accounts {
            count += 1;
            total += account.balance;
        }
        format!("EVM accounts: #{} Total={:?}", count, total)
    }

    pub(crate) fn info_eq(&self, other: &Self) -> String {
        format!("evm: {}", self.accounts == other.accounts)
    }
}
