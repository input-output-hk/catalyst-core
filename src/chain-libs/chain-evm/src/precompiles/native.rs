use super::prelude::Address;
use super::PrecompileOutput;
use super::{EvmPrecompileResult, Precompile};
use evm::{Context, ExitError};

const ERR_TARGET_TOKEN_NOT_FOUND: &str = "Target token not found";

mod costs {
    // Note: not the same type that is defined in machine
    type Gas = u64;

    // TODO(#51): Determine the correct amount of gas
    pub(super) const EXIT_TO_NEAR_GAS: Gas = 0;

    // TODO(#51): Determine the correct amount of gas
    pub(super) const EXIT_TO_ETHEREUM_GAS: Gas = 0;

    // TODO(#51): Determine the correct amount of gas
    pub(super) const FT_TRANSFER_GAS: Gas = 100_000_000_000_000;

    // TODO(#51): Determine the correct amount of gas
    pub(super) const WITHDRAWAL_GAS: Gas = 100_000_000_000_000;
}

pub mod events {
    use crate::precompiles::keccak;
    use crate::precompiles::prelude::{vec, Address, String, ToString, H256, U256};

    /// Derived from event signature (see tests::test_exit_signatures)
    pub const EXIT_TO_NEAR_SIGNATURE: H256 = crate::precompiles::make_h256(
        0x5a91b8bc9c1981673db8fb226dbd8fcd,
        0xd0c23f45cd28abb31403a5392f6dd0c7,
    );
    /// Derived from event signature (see tests::test_exit_signatures)
    pub const EXIT_TO_ETH_SIGNATURE: H256 = crate::precompiles::make_h256(
        0xd046c2bb01a5622bc4b9696332391d87,
        0x491373762eeac0831c48400e2d5a5f07,
    );

    /// The exit precompile events have an `erc20_address` field to indicate
    /// which ERC-20 token is being withdrawn. However, ETH is not an ERC-20 token
    /// So we need to have some other address to fill this field. This constant is
    /// used for this purpose.
    pub const ETH_ADDRESS: Address = crate::precompiles::make_address(0, 0);

    /// ExitToNear(
    ///    Address indexed sender,
    ///    Address indexed erc20_address,
    ///    string indexed dest,
    ///    uint amount
    /// )
    /// Note: in the ERC-20 exit case `sender` == `erc20_address` because it is
    /// the ERC-20 contract which calls the exit precompile. However in the case
    /// of ETH exit the sender will give the true sender (and the `erc20_address`
    /// will not be meaningful because ETH is not an ERC-20 token).
    pub struct ExitToNear {
        pub sender: Address,
        pub erc20_address: Address,
        pub dest: String,
        pub amount: U256,
    }

    impl ExitToNear {
        pub fn encode(self) -> ethabi::RawLog {
            let data = ethabi::encode(&[ethabi::Token::Int(self.amount)]);
            let topics = vec![
                EXIT_TO_NEAR_SIGNATURE,
                encode_address(self.sender),
                encode_address(self.erc20_address),
                keccak(&ethabi::encode(&[ethabi::Token::String(self.dest)])),
            ];

            ethabi::RawLog { topics, data }
        }
    }

    /// ExitToEth(
    ///    Address indexed sender,
    ///    Address indexed erc20_address,
    ///    string indexed dest,
    ///    uint amount
    /// )
    /// Note: in the ERC-20 exit case `sender` == `erc20_address` because it is
    /// the ERC-20 contract which calls the exit precompile. However in the case
    /// of ETH exit the sender will give the true sender (and the `erc20_address`
    /// will not be meaningful because ETH is not an ERC-20 token).
    pub struct ExitToEth {
        pub sender: Address,
        pub erc20_address: Address,
        pub dest: Address,
        pub amount: U256,
    }

    impl ExitToEth {
        pub fn encode(self) -> ethabi::RawLog {
            let data = ethabi::encode(&[ethabi::Token::Int(self.amount)]);
            let topics = vec![
                EXIT_TO_ETH_SIGNATURE,
                encode_address(self.sender),
                encode_address(self.erc20_address),
                encode_address(self.dest),
            ];

            ethabi::RawLog { topics, data }
        }
    }

    fn encode_address(a: Address) -> H256 {
        let mut result = [0u8; 32];
        result[12..].copy_from_slice(a.as_ref());
        H256(result)
    }

    pub fn exit_to_near_schema() -> ethabi::Event {
        ethabi::Event {
            name: "ExitToNear".to_string(),
            inputs: vec![
                ethabi::EventParam {
                    name: "sender".to_string(),
                    kind: ethabi::ParamType::Address,
                    indexed: true,
                },
                ethabi::EventParam {
                    name: "erc20_address".to_string(),
                    kind: ethabi::ParamType::Address,
                    indexed: true,
                },
                ethabi::EventParam {
                    name: "dest".to_string(),
                    kind: ethabi::ParamType::String,
                    indexed: true,
                },
                ethabi::EventParam {
                    name: "amount".to_string(),
                    kind: ethabi::ParamType::Uint(256),
                    indexed: false,
                },
            ],
            anonymous: false,
        }
    }

    pub fn exit_to_eth_schema() -> ethabi::Event {
        ethabi::Event {
            name: "ExitToEth".to_string(),
            inputs: vec![
                ethabi::EventParam {
                    name: "sender".to_string(),
                    kind: ethabi::ParamType::Address,
                    indexed: true,
                },
                ethabi::EventParam {
                    name: "erc20_address".to_string(),
                    kind: ethabi::ParamType::Address,
                    indexed: true,
                },
                ethabi::EventParam {
                    name: "dest".to_string(),
                    kind: ethabi::ParamType::Address,
                    indexed: true,
                },
                ethabi::EventParam {
                    name: "amount".to_string(),
                    kind: ethabi::ParamType::Uint(256),
                    indexed: false,
                },
            ],
            anonymous: false,
        }
    }
}

pub struct ExitToNear; //TransferEthToNear

impl ExitToNear {
    /// Exit to NEAR precompile address
    ///
    /// Address: `0xe9217bc70b7ed1f598ddd3199e80b093fa71124f`
    /// This address is computed as: `&keccak("exitToNear")[12..]`
    pub const ADDRESS: Address =
        super::make_address(0xe9217bc7, 0x0b7ed1f598ddd3199e80b093fa71124f);
}

impl Precompile for ExitToNear {
    fn required_gas(_input: &[u8]) -> Result<u64, ExitError> {
        Ok(costs::EXIT_TO_NEAR_GAS)
    }

    fn run(
        input: &[u8],
        target_gas: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> EvmPrecompileResult {
        if let Some(target_gas) = target_gas {
            if Self::required_gas(input)? > target_gas {
                return Err(ExitError::OutOfGas);
            }
        }

        Ok(PrecompileOutput::default().into())
    }
}

pub struct ExitToEthereum;

impl ExitToEthereum {
    /// Exit to Ethereum precompile address
    ///
    /// Address: `0xb0bd02f6a392af548bdf1cfaee5dfa0eefcc8eab`
    /// This address is computed as: `&keccak("exitToEthereum")[12..]`
    pub const ADDRESS: Address =
        super::make_address(0xb0bd02f6, 0xa392af548bdf1cfaee5dfa0eefcc8eab);
}

impl Precompile for ExitToEthereum {
    fn required_gas(_input: &[u8]) -> Result<u64, ExitError> {
        Ok(costs::EXIT_TO_ETHEREUM_GAS)
    }

    fn run(
        input: &[u8],
        target_gas: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> EvmPrecompileResult {
        if let Some(target_gas) = target_gas {
            if Self::required_gas(input)? > target_gas {
                return Err(ExitError::OutOfGas);
            }
        }

        Ok(PrecompileOutput::default().into())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_exit_signatures() {
        let exit_to_near = super::events::exit_to_near_schema();
        let exit_to_eth = super::events::exit_to_eth_schema();

        assert_eq!(
            exit_to_near.signature(),
            super::events::EXIT_TO_NEAR_SIGNATURE
        );
        assert_eq!(
            exit_to_eth.signature(),
            super::events::EXIT_TO_ETH_SIGNATURE
        );
    }
}
