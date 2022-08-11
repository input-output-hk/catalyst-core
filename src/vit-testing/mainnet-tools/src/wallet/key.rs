#[derive(Debug, Clone)]
pub struct MainnetKey {
    payment_skey_cbor_hex: String,
    payment_vkey_cbor_hex: String,
    stake_skey_cbor_hex: String,
    stake_vkey_cbor_hex: String,
}

impl MainnetKey {
    pub fn payment_skey_cbor_hex(&self) -> String {
        self.payment_skey_cbor_hex.clone()
    }
    pub fn payment_vkey_cbor_hex(&self) -> String {
        self.payment_vkey_cbor_hex.clone()
    }
    pub fn stake_skey_cbor_hex(&self) -> String {
        self.stake_skey_cbor_hex.clone()
    }
    pub fn stake_vkey_cbor_hex(&self) -> String {
        self.stake_vkey_cbor_hex.clone()
    }
}

impl Default for MainnetKey {
    fn default() -> Self {
        Self {
            payment_skey_cbor_hex:
                "58207d06d501ae1302b4765c0a515d6165bb0129817b8e07b8883a66a00f3fcbf41e".to_string(),
            payment_vkey_cbor_hex:
                "5820685c2b477d207253f8587194bebf7d580224f3a2ce2c7a8d100cc0ebfd0a91e0".to_string(),
            stake_skey_cbor_hex:
                "582002157d5a96a9d0e6c69ec05974f0da4ab91ccb3a285ceaae6aa61bd86e5d0549".to_string(),
            stake_vkey_cbor_hex:
                "5820810cace3b75583fbc1d1841c6c8dd5a495cb6a577a3a6e22409e128dbbafc439".to_string(),
        }
    }
}
