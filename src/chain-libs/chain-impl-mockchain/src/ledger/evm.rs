use crate::chaineval::HeaderContentEvalContext;
use crate::evm::EvmTransaction;
use crate::header::BlockDate;
use crate::ledger::Error;
use chain_evm::{
    machine::{
        transact_call, transact_create, transact_create2, BlockHash, BlockNumber, BlockTimestamp,
        Config, Environment, EvmState, Log, VirtualMachine,
    },
    state::{Account, AccountTrie, LogsState},
    Address,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Ledger {
    pub(crate) accounts: AccountTrie,
    pub(crate) logs: LogsState,
    pub(crate) environment: Environment,
    pub(crate) current_epoch: BlockEpoch,
}

impl Default for Ledger {
    fn default() -> Self {
        Ledger::new()
    }
}

impl EvmState for super::Ledger {
    fn environment(&self) -> &Environment {
        &self.evm.environment
    }

    fn account(&self, address: Address) -> Option<Account> {
        self.evm.accounts.get(&address).cloned()
    }

    fn contains(&self, address: Address) -> bool {
        self.evm.accounts.contains(&address)
    }

    fn modify_account<F>(&mut self, address: Address, f: F)
    where
        F: FnOnce(Account) -> Option<Account>,
    {
        self.evm.accounts = self.evm.accounts.clone().modify_account(address, f);
    }

    fn update_logs(&mut self, block_hash: BlockHash, logs: Vec<Log>) {
        self.evm.logs.put(block_hash, logs);
    }
}

impl super::Ledger {
    pub fn run_transaction(
        &mut self,
        contract: EvmTransaction,
        config: Config,
    ) -> Result<(), Error> {
        let config = config.into();
        match contract {
            EvmTransaction::Create {
                caller,
                value,
                init_code,
                gas_limit,
                access_list,
            } => {
                let vm = VirtualMachine::new(self, &config, caller, gas_limit, true);
                transact_create(vm, value, init_code, access_list)?;
            }
            EvmTransaction::Create2 {
                caller,
                value,
                init_code,
                salt,
                gas_limit,
                access_list,
            } => {
                let vm = VirtualMachine::new(self, &config, caller, gas_limit, true);
                transact_create2(vm, value, init_code, salt, access_list)?;
            }
            EvmTransaction::Call {
                caller,
                address,
                value,
                data,
                gas_limit,
                access_list,
            } => {
                let vm = VirtualMachine::new(self, &config, caller, gas_limit, true);
                let _byte_code_msg = transact_call(vm, address, value, data, access_list)?;
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BlockEpoch {
    epoch: u32,
    epoch_start: BlockTimestamp,
    slots_per_epoch: u32,
    slot_duration: u8,
}

impl Ledger {
    /// Updates block values for EVM environment
    pub fn update_block_environment(
        &mut self,
        metadata: &HeaderContentEvalContext,
        slots_per_epoch: u32,
        slot_duration: u8,
    ) {
        // use content hash from the apply block as the EVM block hash
        let next_hash: BlockHash = <[u8; 32]>::from(metadata.block_id).into();
        self.environment.block_hashes.insert(0, next_hash);
        self.environment.block_number = BlockNumber::from(self.environment.block_hashes.len());
        self.update_block_timestamp(metadata.block_date, slots_per_epoch, slot_duration);
    }
    /// Updates the block timestamp for EVM environment
    fn update_block_timestamp(
        &mut self,
        block_date: BlockDate,
        slots_per_epoch: u32,
        slot_duration: u8,
    ) {
        let BlockDate {
            epoch: this_epoch,
            slot_id,
        } = block_date;

        // update block epoch if needed
        if this_epoch > self.current_epoch.epoch {
            let BlockEpoch {
                epoch: _,
                epoch_start,
                slots_per_epoch: spe,
                slot_duration: sdur,
            } = self.current_epoch;
            let new_epoch = this_epoch;
            let new_epoch_start = epoch_start + spe as u64 * sdur as u64;
            self.current_epoch = BlockEpoch {
                epoch: new_epoch,
                epoch_start: new_epoch_start,
                slots_per_epoch,
                slot_duration,
            };
        }

        // calculate the U256 value from epoch and slot parameters
        let block_timestamp = self.current_epoch.epoch_start
            + slot_id as u64 * self.current_epoch.slot_duration as u64;
        // update EVM enviroment
        self.environment.block_timestamp = block_timestamp;
    }
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            accounts: Default::default(),
            logs: Default::default(),
            environment: Environment {
                gas_price: Default::default(),
                chain_id: Default::default(),
                block_hashes: Default::default(),
                block_number: Default::default(),
                block_coinbase: Default::default(),
                block_timestamp: Default::default(),
                block_difficulty: Default::default(),
                block_gas_limit: Default::default(),
                block_base_fee_per_gas: Default::default(),
            },
            current_epoch: BlockEpoch {
                epoch: 0,
                epoch_start: BlockTimestamp::default(),
                slots_per_epoch: 1,
                slot_duration: 10,
            },
        }
    }
}

impl Ledger {
    pub(crate) fn stats(&self) -> String {
        let Ledger { accounts, .. } = self;
        let mut count = 0;
        for (_, _) in accounts {
            count += 1;
        }
        format!("EVM accounts: #{}", count)
    }

    pub(crate) fn info_eq(&self, other: &Self) -> String {
        format!("evm: {}", self.accounts == other.accounts)
    }
}
