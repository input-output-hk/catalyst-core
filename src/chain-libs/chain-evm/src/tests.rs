use crate::machine::test::TestEvmState;
use crate::machine::{transact_call, VirtualMachine};
use crate::{state::Account, Config};
use ethereum_types::{H160, U256};
use evm_test_suite::{AccountState, BlockHeader, CallTransaction, NetworkType};
use std::collections::BTreeSet;

struct TestEvmLedger {
    state: TestEvmState,
    config: Config,
    coinbase_addresses: BTreeSet<H160>,
}

impl TryFrom<AccountState> for Account {
    type Error = String;
    fn try_from(val: AccountState) -> Result<Self, Self::Error> {
        Ok(Self {
            nonce: val.nonce,
            balance: val
                .balance
                .try_into()
                .map_err(|_| "can not convert balance")?,
            storage: val.storage.into_iter().collect(),
            code: val.code,
        })
    }
}

impl evm_test_suite::TestEvmState for TestEvmLedger {
    fn init_state() -> Self {
        Self {
            state: TestEvmState {
                environment: Default::default(),
                accounts: Default::default(),
                logs: Default::default(),
            },
            config: Default::default(),
            coinbase_addresses: Default::default(),
        }
    }

    fn validate_account(
        &self,
        address: H160,
        expected_account: AccountState,
    ) -> Result<(), String> {
        if !self.coinbase_addresses.contains(&address) {
            let account = self
                .state
                .accounts
                .get(&address)
                .ok_or("Can not find account")?;

            let expected_account = expected_account.try_into()?;

            if &expected_account != account {
                let storage_info = |account: &Account| {
                    let mut storage = "{".to_string();
                    for (key, value) in account.storage.iter() {
                        storage = format!("{} |key: {} , value: {}| ", storage, key, value);
                    }
                    format!("{}}}", storage)
                };

                let expected_storage = storage_info(&expected_account);
                let account_storage = storage_info(account);

                Err(format!(
                    "Account mismatch,
                    address: {},
                    current: {{ balance: {}, nonce: {}, code: {}, storage: {} }},
                    expected: {{ balance: {}, nonce: {}, code: {}, storage: {} }}",
                    address,
                    account.balance,
                    account.nonce,
                    hex::encode(&account.code),
                    account_storage,
                    expected_account.balance,
                    expected_account.nonce,
                    hex::encode(expected_account.code),
                    expected_storage
                ))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn try_apply_chain_id(mut self, id: U256) -> Result<Self, String> {
        self.state.environment.chain_id = id;
        Ok(self)
    }

    fn try_apply_network_type(mut self, net_type: NetworkType) -> Result<Self, String> {
        match net_type {
            NetworkType::Berlin => self.config = Config::Berlin,
            NetworkType::Istanbul => self.config = Config::Istanbul,
            NetworkType::London => self.config = Config::London,
        }
        Ok(self)
    }

    fn try_apply_account(mut self, address: H160, account: AccountState) -> Result<Self, String> {
        self.state.accounts = self.state.accounts.put(address, account.try_into()?);
        Ok(self)
    }

    fn try_apply_block_header(mut self, block_header: BlockHeader) -> Result<Self, String> {
        self.state.environment.block_gas_limit = block_header.gas_limit;
        self.state.environment.block_number = block_header.number;
        self.state.environment.block_timestamp = block_header.timestamp;
        self.state.environment.block_difficulty = block_header.difficulty;
        self.state.environment.block_coinbase = block_header.coinbase;

        self.state.environment.block_hashes.push(block_header.hash);

        self.coinbase_addresses.insert(block_header.coinbase);

        Ok(self)
    }

    fn try_apply_transaction(mut self, tx: CallTransaction) -> Result<Self, String> {
        self.state.environment.gas_price = tx.gas_price;
        let config = self.config.into();
        let vm = VirtualMachine::new(
            &mut self.state,
            &config,
            tx.sender,
            tx.gas_limit.as_u64(),
            true,
        );
        transact_call(vm, tx.to, tx.value, tx.data, Vec::new())
            .map_err(|e| format!("can not run transaction, err: {}", e))?;

        Ok(self)
    }
}

// This was left for the convinience to run and debug a separate test case
#[test]
#[ignore]
fn run_evm_test() {
    evm_test_suite::run_evm_test::<TestEvmLedger>(evm_test_suite::arithmetic::ADD);
}

#[test]
fn run_evm_tests() {
    evm_test_suite::run_evm_tests::<TestEvmLedger>();
}
