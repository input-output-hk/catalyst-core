use crate::{
    transaction::{EthereumSignedTransaction, EthereumUnsignedTransaction},
    Address,
};
use ethereum_types::H256;
use secp256k1::{
    ecdsa::RecoverableSignature, rand::rngs::ThreadRng, KeyPair, Message, PublicKey, Secp256k1,
    SecretKey,
};
use sha3::{Digest, Keccak256};

/// Generate new SECP256K1 keypair.
pub fn generate_keypair() -> KeyPair {
    let secp = Secp256k1::new();
    let mut rng = ThreadRng::default();
    KeyPair::new(&secp, &mut rng)
}

/// A secret key for an Ethereum account.
pub struct Secret(SecretKey);

impl Secret {
    pub fn from_hash(data: &H256) -> Self {
        Self(SecretKey::from_slice(data.as_fixed_bytes()).unwrap())
    }

    pub fn from_slice(data: &[u8]) -> Result<Self, secp256k1::Error> {
        Ok(Self(SecretKey::from_slice(data)?))
    }

    pub fn seckey(&self) -> &SecretKey {
        &self.0
    }

    pub fn secret_hash(&self) -> H256 {
        H256::from_slice(&self.0.secret_bytes())
    }

    pub fn address(&self) -> Address {
        let pubkey_bytes = PublicKey::from_secret_key_global(&self.0).serialize_uncompressed();
        Address::from_slice(&Keccak256::digest(&pubkey_bytes[1..]).as_slice()[12..])
    }
    pub fn sign(
        &self,
        tx: EthereumUnsignedTransaction,
    ) -> Result<EthereumSignedTransaction, secp256k1::Error> {
        tx.sign(self)
    }
}

impl From<&KeyPair> for Secret {
    fn from(other: &KeyPair) -> Self {
        Secret(SecretKey::from_keypair(other))
    }
}

/// Generate a secret key for an Ethereum account from the global context.
pub fn generate_account_secret() -> Secret {
    let keypair = generate_keypair();
    Secret::from(&keypair)
}

/// Sign a given hash of data with a secret key.
pub fn sign_data_hash(
    tx_hash: &H256,
    secret: &Secret,
) -> Result<RecoverableSignature, secp256k1::Error> {
    let s = Secp256k1::new();
    let h = Message::from_slice(tx_hash.as_fixed_bytes())?;
    let secret = secret.seckey();
    Ok(s.sign_ecdsa_recoverable(&h, secret))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethereum::{
        AccessListItem, EIP1559Transaction, EIP2930Transaction, LegacyTransaction,
        LegacyTransactionMessage, TransactionAction, TransactionSignature, TransactionV2,
    };
    use ethereum_types::{H160, U256};
    use secp256k1::PublicKey;
    use std::str::FromStr;

    // chain-id's listed in https://eips.ethereum.org/EIPS/eip-155
    const TEST_CHAIN_ID: u64 = 1;

    #[test]
    fn can_decode_raw_transaction() {
        let bytes = hex::decode("f901e48080831000008080b90196608060405234801561001057600080fd5b50336000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055507fc68045c3c562488255b55aa2c4c7849de001859ff0d8a36a75c2d5ed80100fb660405180806020018281038252600d8152602001807f48656c6c6f2c20776f726c64210000000000000000000000000000000000000081525060200191505060405180910390a160cf806100c76000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c80638da5cb5b14602d575b600080fd5b60336075565b604051808273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200191505060405180910390f35b6000809054906101000a900473ffffffffffffffffffffffffffffffffffffffff168156fea265627a7a72315820fae816ad954005c42bea7bc7cb5b19f7fd5d3a250715ca2023275c9ca7ce644064736f6c634300050f003278a04cab43609092a99cf095d458b61b47189d1bbab64baed10a0fd7b7d2de2eb960a011ab1bcda76dfed5e733219beb83789f9887b2a7b2e61759c7c90f7d40403201").unwrap();

        assert!(EthereumSignedTransaction::from_bytes(bytes.as_slice()).is_ok());
    }

    #[test]
    fn legacy_transaction() {
        let tx = EthereumSignedTransaction(TransactionV2::Legacy(LegacyTransaction {
			nonce: 12_u64.into(),
			gas_price: 20_000_000_000_u64.into(),
			gas_limit: 21000_u64.into(),
			action: TransactionAction::Call(
				H160::from_slice(hex::decode("727fc6a68321b754475c668a6abfb6e9e71c169a").unwrap().as_slice()),
			),
			value: (10_u128 * 1_000_000_000_u128 * 1_000_000_000_u128).into(),
			input: hex::decode("a9059cbb000000000213ed0f886efd100b67c7e4ec0a85a7d20dc971600000000000000000000015af1d78b58c4000").unwrap(),
			signature: TransactionSignature::new(38, H256::from_slice(hex::decode("be67e0a07db67da8d446f76add590e54b6e92cb6b8f9835aeb67540579a27717").unwrap().as_slice()),  H256::from_slice(hex::decode("2d690516512020171c1ec870f6ff45398cc8609250326be89915fb538e7bd718").unwrap().as_slice())).unwrap(),
		}));

        assert_eq!(
            rlp::decode::<EthereumSignedTransaction>(&rlp::encode(&tx)).unwrap(),
            tx,
        );
    }

    #[test]
    fn eip2930_transaction() {
        let tx =
            EthereumSignedTransaction(TransactionV2::EIP2930(EIP2930Transaction {
                chain_id: 5,
                nonce: 7_u64.into(),
                gas_price: 30_000_000_000_u64.into(),
                gas_limit: 5_748_100_u64.into(),
                action: TransactionAction::Call(H160::from_slice(
                    hex::decode("811a752c8cd697e3cb27279c330ed1ada745a8d7")
                        .unwrap()
                        .as_slice(),
                )),
                value: (2_u128 * 1_000_000_000_u128 * 1_000_000_000_u128).into(),
                input: hex::decode("6ebaf477f83e051589c1188bcc6ddccd").unwrap(),
                access_list: vec![
                    AccessListItem {
                        address: H160::from_slice(
                            hex::decode("de0b295669a9fd93d5f28d9ec85e40f4cb697bae")
                                .unwrap()
                                .as_slice(),
                        ),
                        storage_keys: vec![
                        H256::from_slice(hex::decode(
                            "0000000000000000000000000000000000000000000000000000000000000003",
                        ).unwrap().as_slice()),
                        H256::from_slice(hex::decode(
                            "0000000000000000000000000000000000000000000000000000000000000007",
                        )
                        .unwrap().as_slice()),
                    ],
                    },
                    AccessListItem {
                        address: H160::from_slice(
                            hex::decode("bb9bc244d798123fde783fcc1c72d3bb8c189413")
                                .unwrap()
                                .as_slice(),
                        ),
                        storage_keys: vec![],
                    },
                ],
                odd_y_parity: false,
                r: H256::from_slice(
                    hex::decode("36b241b061a36a32ab7fe86c7aa9eb592dd59018cd0443adc0903590c16b02b0")
                        .unwrap()
                        .as_slice(),
                ),
                s: H256::from_slice(
                    hex::decode("5edcc541b4741c5cc6dd347c5ed9577ef293a62787b4510465fadbfe39ee4094")
                        .unwrap()
                        .as_slice(),
                ),
            }));

        assert_eq!(
            rlp::decode::<EthereumSignedTransaction>(&rlp::encode(&tx)).unwrap(),
            tx,
        );
    }

    #[test]
    fn eip1559_transaction() {
        let tx = EthereumSignedTransaction(TransactionV2::EIP1559(EIP1559Transaction {
            chain_id: 5,
            nonce: 7_u64.into(),
            max_priority_fee_per_gas: 10_000_000_000_u64.into(),
            max_fee_per_gas: 30_000_000_000_u64.into(),
            gas_limit: 5_748_100_u64.into(),
            action: TransactionAction::Call(H160::from_slice(
                hex::decode("811a752c8cd697e3cb27279c330ed1ada745a8d7")
                    .unwrap()
                    .as_slice(),
            )),
            value: (2_u128 * 1_000_000_000_u128 * 1_000_000_000_u128).into(),
            input: hex::decode("6ebaf477f83e051589c1188bcc6ddccd").unwrap(),
            access_list: vec![
                AccessListItem {
                    address: H160::from_slice(
                        hex::decode("de0b295669a9fd93d5f28d9ec85e40f4cb697bae")
                            .unwrap()
                            .as_slice(),
                    ),
                    storage_keys: vec![
                        H256::from_slice(
                            hex::decode(
                                "0000000000000000000000000000000000000000000000000000000000000003",
                            )
                            .unwrap()
                            .as_slice(),
                        ),
                        H256::from_slice(
                            hex::decode(
                                "0000000000000000000000000000000000000000000000000000000000000007",
                            )
                            .unwrap()
                            .as_slice(),
                        ),
                    ],
                },
                AccessListItem {
                    address: H160::from_slice(
                        hex::decode("bb9bc244d798123fde783fcc1c72d3bb8c189413")
                            .unwrap()
                            .as_slice(),
                    ),
                    storage_keys: vec![],
                },
            ],
            odd_y_parity: false,
            r: H256::from_slice(
                hex::decode("36b241b061a36a32ab7fe86c7aa9eb592dd59018cd0443adc0903590c16b02b0")
                    .unwrap()
                    .as_slice(),
            ),
            s: H256::from_slice(
                hex::decode("5edcc541b4741c5cc6dd347c5ed9577ef293a62787b4510465fadbfe39ee4094")
                    .unwrap()
                    .as_slice(),
            ),
        }));

        assert_eq!(
            rlp::decode::<EthereumSignedTransaction>(&rlp::encode(&tx)).unwrap(),
            tx,
        );
    }

    #[test]
    fn test_transaction_signature_is_recoverable() {
        let keypair = generate_keypair();
        let unsigned_tx =
            crate::transaction::EthereumUnsignedTransaction::Legacy(LegacyTransactionMessage {
                nonce: U256::zero(),
                gas_price: U256::zero(),
                gas_limit: U256::zero(),
                action: ethereum::TransactionAction::Create,
                value: U256::zero(),
                input: Vec::new(),
                chain_id: Some(TEST_CHAIN_ID),
            });
        let tx_hash = unsigned_tx.hash();
        let secret = Secret::from_slice(&keypair.secret_bytes()).unwrap();
        let signature = sign_data_hash(&tx_hash, &secret).unwrap();

        let msg = Message::from_slice(tx_hash.as_fixed_bytes()).unwrap();
        let pubkey = PublicKey::from_secret_key_global(secret.seckey());

        assert_eq!(signature.recover(&msg), Ok(pubkey))
    }

    #[test]
    fn test_legacy_transaction_signature() {
        // This test takes values found at https://eips.ethereum.org/EIPS/eip-155#example
        let unsigned_tx =
            crate::transaction::EthereumUnsignedTransaction::Legacy(LegacyTransactionMessage {
                nonce: U256::from(9_u64),
                gas_price: U256::from(20_u64 * 10_u64.pow(9)),
                gas_limit: U256::from(21_000_u64),
                action: ethereum::TransactionAction::Call(
                    Address::from_str("0x3535353535353535353535353535353535353535").unwrap(),
                ),
                value: U256::from(10u64.pow(18)),
                input: Vec::new(),
                chain_id: Some(TEST_CHAIN_ID),
            });

        // test signing hash
        let tx_hash = unsigned_tx.hash();
        assert_eq!(
            format!("{:x}", tx_hash),
            "daf5a779ae972f972197303d7b574746c7ef83eadac0f2791ad23db92e4c8e53"
        );

        // given a secret key
        let secret = Secret::from_slice(&[0x46; 32]).unwrap();

        // test signed transaction
        let signed = unsigned_tx.sign(&secret).unwrap();
        assert_eq!(
            hex::encode(signed.to_bytes().as_slice()),
            "f86c098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a76400008025a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83"
        );
    }

    #[test]
    fn test_recovery_of_legacy_signed_transaction() {
        let raw_signed_tx = "f86c098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a76400008025a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83";
        let signed =
            EthereumSignedTransaction::from_bytes(&hex::decode(raw_signed_tx).unwrap()).unwrap();
        // given the signing secret key's address
        let caller = Address::from_str("0x9d8a62f656a8d1615c1294fd71e9cfb3e4855a4f").unwrap();
        assert_eq!(signed.recover().unwrap(), caller);
    }

    #[test]
    fn test_decoding_rlp_legacy_signed_transaction() {
        let raw_signed_tx = "f86c098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a76400008025a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83";
        let signed =
            EthereumSignedTransaction::from_bytes(&hex::decode(raw_signed_tx).unwrap()).unwrap();
        assert_eq!(
            hex::encode(signed.to_bytes().as_slice()),
            "f86c098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a76400008025a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83"
        );
    }

    #[test]
    fn account_secret_has_valid_address() {
        // example taken from `test_legacy_transaction_signature` secret
        //address: "0x9d8a62f656a8d1615c1294fd71e9cfb3e4855a4f",
        //privateKey: "0x4646464646464646464646464646464646464646464646464646464646464646",
        let secret = Secret::from_slice(&[0x46; 32]).unwrap();
        assert_eq!(
            secret.address(),
            Address::from_str("0x9d8a62f656a8d1615c1294fd71e9cfb3e4855a4f").unwrap()
        );
        // example taken from https://web3js.readthedocs.io/en/v1.7.3/web3-eth-accounts.html#example
        //address: "0xb8CE9ab6943e0eCED004cDe8e3bBed6568B2Fa01",
        //privateKey: "0x348ce564d427a3311b6536bbcff9390d69395b06ed6c486954e971d960fe8709",
        let mut secret_bytes = [0u8; 32];
        hex::decode_to_slice(
            "348ce564d427a3311b6536bbcff9390d69395b06ed6c486954e971d960fe8709",
            &mut secret_bytes,
        )
        .unwrap();
        let secret = Secret::from_slice(&secret_bytes).unwrap();
        assert_eq!(
            secret.address(),
            Address::from_str("b8CE9ab6943e0eCED004cDe8e3bBed6568B2Fa01").unwrap()
        )
    }
}
