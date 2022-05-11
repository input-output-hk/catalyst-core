use crate::{
    transaction::{EthereumSignedTransaction, EthereumTransaction},
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
    pub fn from_hash(data: &H256) -> Result<Self, secp256k1::Error> {
        Ok(Self(SecretKey::from_slice(data.as_fixed_bytes())?))
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
        tx: EthereumTransaction,
    ) -> Result<EthereumSignedTransaction, secp256k1::Error> {
        tx.sign(&self.secret_hash())
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
    use ethereum::LegacyTransactionMessage;
    use ethereum_types::U256;
    use secp256k1::PublicKey;
    use std::str::FromStr;

    // chain-id's listed in https://eips.ethereum.org/EIPS/eip-155
    const TEST_CHAIN_ID: u64 = 1;

    #[test]
    fn test_transaction_signature_is_recoverable() {
        let keypair = generate_keypair();
        let unsigned_tx =
            crate::transaction::EthereumTransaction::Legacy(LegacyTransactionMessage {
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
            crate::transaction::EthereumTransaction::Legacy(LegacyTransactionMessage {
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

        // test signing data
        assert_eq!(
            hex::encode(unsigned_tx.to_bytes().as_slice()),
            "ec098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a764000080018080"
        );

        // test signing hash
        let tx_hash = unsigned_tx.hash();
        assert_eq!(
            format!("{:x}", tx_hash),
            "daf5a779ae972f972197303d7b574746c7ef83eadac0f2791ad23db92e4c8e53"
        );

        // given a secret key
        let secret = Secret::from_slice(&[0x46; 32]).unwrap();

        // test signed transaction
        let signed = unsigned_tx.sign(&secret.secret_hash()).unwrap();
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
