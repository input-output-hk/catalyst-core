use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io::BufReader;
use std::mem::size_of;
use std::path::PathBuf;
use std::str::FromStr;

use chain_evm::{
    primitive_types::{H160, H256, U256},
    state::{Account, Trie},
    Address, Config,
};

use crate::evm::EvmTransaction;
use crate::ledger::evm::Ledger;

struct TestEvmState {
    ledger: Ledger,
    config: Config,
    coinbase_addresses: BTreeSet<String>,
}

impl TestEvmState {
    fn new() -> Self {
        Self {
            ledger: Default::default(),
            config: Default::default(),
            coinbase_addresses: Default::default(),
        }
    }
}

impl TestEvmState {
    fn set_evm_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    fn set_account(mut self, address: Address, account: Account) -> Self {
        self.ledger.accounts = self.ledger.accounts.put(address, account);
        self
    }

    fn set_chain_id(mut self, chain_id: U256) -> Self {
        self.ledger.environment.chain_id = chain_id;
        self
    }
}

impl TestEvmState {
    fn try_apply_network(self, network: String) -> Result<Self, String> {
        println!("Network type: {}", network);
        match network.as_str() {
            "Istanbul" => Ok(self.set_evm_config(Config::Istanbul)),
            "Berlin" => Ok(self.set_evm_config(Config::Berlin)),
            "London" => Ok(self.set_evm_config(Config::London)),
            network => Err(format!("Not known network type, {}", network)),
        }
    }

    fn try_apply_account(self, address: String, account: TestAccountState) -> Result<Self, String> {
        Ok(self.set_account(
            H160::from_str(&address).map_err(|_| "Can not parse address")?,
            account.try_into()?,
        ))
    }

    fn try_apply_accounts<I>(mut self, iter: I) -> Result<Self, String>
    where
        I: Iterator<Item = (String, TestAccountState)>,
    {
        for (address, account) in iter {
            self = self.try_apply_account(address, account)?;
        }
        Ok(self)
    }

    fn try_apply_block_header(mut self, block_header: TestBlockHeader) -> Result<Self, String> {
        self.ledger.environment.block_gas_limit =
            U256::from_str(&block_header.gas_limit).map_err(|_| "Can not parse gas limit")?;
        self.ledger.environment.block_number =
            U256::from_str(&block_header.number).map_err(|_| "Can not parse number")?;
        self.ledger.environment.block_timestamp =
            U256::from_str(&block_header.timestamp).map_err(|_| "Can not parse timestamp")?;
        self.ledger.environment.block_difficulty =
            U256::from_str(&block_header.difficulty).map_err(|_| "Can not parse difficulty")?;

        self.ledger
            .environment
            .block_hashes
            .push(H256::from_str(&block_header.hash).expect("Can not parse hash"));

        self.coinbase_addresses.insert(block_header.coinbase);

        Ok(self)
    }

    fn try_apply_transaction(mut self, tx: TestEvmTransaction) -> Result<Self, String> {
        let gas_price =
            U256::from_str(&tx.gas_price).map_err(|_| "Can not parse transaction gas limit")?;

        self.ledger.environment.gas_price = gas_price;

        self.ledger
            .run_transaction(tx.try_into()?, self.config)
            .map_err(|e| format!("can not run transaction, err: {}", e))?;

        Ok(self)
    }

    fn try_apply_block(mut self, block: TestBlock) -> Result<Self, String> {
        self = self.try_apply_block_header(block.block_header)?;
        for transaction in block.transactions {
            self = self.try_apply_transaction(transaction)?;
        }

        Ok(self)
    }

    fn try_apply_blocks<I>(mut self, iter: I) -> Result<Self, String>
    where
        I: Iterator<Item = TestBlock>,
    {
        for block in iter {
            self = self.try_apply_block(block)?;
        }
        Ok(self)
    }
}

impl TestEvmState {
    fn validate_accounts<I>(&self, iter: I) -> Result<(), String>
    where
        I: Iterator<Item = (String, TestAccountState)>,
    {
        for (address, account) in iter {
            self.validate_account(address, account)?;
        }
        Ok(())
    }

    fn validate_account(
        &self,
        address: String,
        expected_state: TestAccountState,
    ) -> Result<(), String> {
        // skip coinbase accounts
        if !self.coinbase_addresses.contains(&address) {
            let account = self
                .ledger
                .accounts
                .get(&H160::from_str(&address).map_err(|_| "Can not parse address")?)
                .ok_or("Can not find account")?;
            let expected_account: Account = expected_state.try_into()?;

            if &expected_account != account {
                Err(format!(
                    "Account mismatch,
                    address: {},
                    current: {{ balance: {}, nonce: {}, code: {} }},
                    expected: {{ balance: {}, nonce: {}, code: {} }}",
                    address,
                    account.balance,
                    account.nonce,
                    hex::encode(&account.code),
                    expected_account.balance,
                    expected_account.nonce,
                    hex::encode(expected_account.code),
                ))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

#[derive(Deserialize)]
struct TestAccountState {
    balance: String,
    code: String,
    nonce: String,
    storage: BTreeMap<String, String>,
}

impl TryFrom<TestAccountState> for Account {
    type Error = String;
    fn try_from(account: TestAccountState) -> Result<Self, Self::Error> {
        let mut storage = Trie::default();
        for (k, v) in account.storage {
            let feel_zeros = |mut val: String| -> Result<String, String> {
                val = val[0..2]
                    .eq("0x")
                    .then(|| val[2..].to_string())
                    .ok_or("Missing '0x' prefix for hex data")?;

                while val.len() < size_of::<H256>() * 2 {
                    val = "00".to_string() + &val;
                }
                val = "0x".to_string() + &val;
                Ok(val)
            };
            storage = storage.put(
                H256::from_str(&feel_zeros(k)?).map_err(|_| "Can not parse account storage key")?,
                H256::from_str(&feel_zeros(v)?).map_err(|_| "Can not parse account storage key")?,
            );
        }
        Ok(Self {
            nonce: U256::from_str(&account.nonce).map_err(|_| "Can not parse nonce")?,
            balance: U256::from_str(&account.balance).map_err(|_| "Can not parse balance")?,
            storage,
            code: hex::decode(
                account.code[0..2]
                    .eq("0x")
                    .then(|| account.code[2..].to_string())
                    .expect("Missing '0x' prefix for hex data"),
            )
            .map_err(|_| "Can not parse code")?,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestEvmTransaction {
    data: String,
    gas_limit: String,
    gas_price: String,
    sender: String,
    to: String,
    value: String,
}

impl TryFrom<TestEvmTransaction> for EvmTransaction {
    type Error = String;
    fn try_from(tx: TestEvmTransaction) -> Result<Self, Self::Error> {
        let gas_limit = U256::from_str(&tx.gas_limit)
            .map_err(|_| "Can not parse transaction gas limit")?
            .as_u64();
        let value = U256::from_str(&tx.value).map_err(|_| "Can not parse transaction value")?;
        let data = hex::decode(
            tx.data[0..2]
                .eq("0x")
                .then(|| tx.data[2..].to_string())
                .expect("Missing '0x' prefix for hex data"),
        )
        .map_err(|_| "Can not parse transaction data")?;
        let caller = H160::from_str(&tx.sender).map_err(|_| "Can not parse transaction sender")?;
        let address = H160::from_str(&tx.to).map_err(|_| "Can not parse transaction to")?;

        Ok(Self::Call {
            address,
            gas_limit,
            value,
            data,
            caller,
            access_list: Vec::new(),
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestBlockHeader {
    coinbase: String,
    difficulty: String,
    gas_limit: String,
    hash: String,
    number: String,
    timestamp: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestBlock {
    block_header: TestBlockHeader,
    transactions: Vec<TestEvmTransaction>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestCase {
    pre: BTreeMap<String, TestAccountState>,
    network: String,
    genesis_block_header: TestBlockHeader,
    blocks: Vec<TestBlock>,
    post_state: BTreeMap<String, TestAccountState>,
}

pub fn run_evm_test(path: PathBuf) {
    println!(
        "\n----------- Running tests: {} -----------",
        path.file_name().unwrap().to_str().unwrap()
    );

    let file = std::fs::File::open(path).expect("Open file failed");
    let reader = BufReader::new(file);

    let test: BTreeMap<String, TestCase> =
        serde_json::from_reader(reader).expect("Parse test cases failed");

    for (test_name, test_case) in test {
        println!("\nRunning test case: {} ...", test_name);
        let evm_state_builder = TestEvmState::new()
            .set_chain_id(U256::from_str("0xff").unwrap())
            .try_apply_network(test_case.network)
            .unwrap()
            .try_apply_accounts(test_case.pre.into_iter())
            .unwrap()
            .try_apply_block_header(test_case.genesis_block_header)
            .unwrap()
            .try_apply_blocks(test_case.blocks.into_iter())
            .unwrap();

        evm_state_builder
            .validate_accounts(test_case.post_state.into_iter())
            .unwrap();
    }
}

// TODO: need to fix following tests
// "../evm-tests/BlockchainTests/GeneralStateTests/VMTests/vmTests/blockInfo.json"
// "../evm-tests/BlockchainTests/GeneralStateTests/VMTests/vmTests/envInfo.json"
// "../evm-tests/BlockchainTests/GeneralStateTests/VMTests/vmIOandFlowOperations/loop_stacklimit.json"
// "../evm-tests/BlockchainTests/GeneralStateTests/VMTests/vmIOandFlowOperations/jumpToPush.json"
#[test]
#[ignore]
fn run_evm_tests() {
    let vm_tests_dir = std::fs::read_dir("../evm-tests/BlockchainTests/GeneralStateTests/VMTests")
        .expect("Can not find vm tests directory");

    for vm_test_dir in vm_tests_dir {
        let vm_test_dir = vm_test_dir.expect("Can not open vm tests dir entry");
        println!(
            "Running {} tests ...",
            vm_test_dir.file_name().to_str().unwrap()
        );

        if vm_test_dir.file_name().to_str().unwrap() == "vmPerformance" {
            println!("Skipping");
            continue;
        }

        for vm_test in std::fs::read_dir(vm_test_dir.path()).unwrap() {
            let vm_test = vm_test.expect("Can not open vm test entry");
            run_evm_test(vm_test.path());
        }
    }
}

// This was left for the convinience to run and debug a separate test case
#[test]
#[ignore]
fn evm_test() {
    run_evm_test(PathBuf::from(
        "../evm-tests/BlockchainTests/GeneralStateTests/VMTests/vmTests/blockInfo.json",
    ));
}
