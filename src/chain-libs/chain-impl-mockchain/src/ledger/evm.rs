use crate::evm::EvmTransaction;
use crate::ledger::Error;
use chain_evm::{
    machine::{Config, Environment, VirtualMachine},
    state::{AccountTrie, Balance, LogsState},
};

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Ledger {
    pub(crate) accounts: AccountTrie,
    pub(crate) logs: LogsState,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            accounts: Default::default(),
            logs: Default::default(),
        }
    }
    pub fn run_transaction<'runtime>(
        &mut self,
        contract: EvmTransaction,
        config: &'runtime Config,
        environment: &'runtime Environment,
    ) -> Result<(), Error> {
        let mut vm = self.virtual_machine(config, environment);
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

    pub(crate) fn virtual_machine<'runtime>(
        &self,
        config: &'runtime Config,
        environment: &'runtime Environment,
    ) -> VirtualMachine<'runtime> {
        VirtualMachine::new_with_state(config, environment, self.accounts.clone())
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
