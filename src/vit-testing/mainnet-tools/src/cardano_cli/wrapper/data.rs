#![allow(dead_code)]

use super::error::Error;
use crate::cardano_cli::wrapper::utils::write_content;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct CardanoKeyTemplate {
    r#type: String,
    description: String,
    #[serde(rename = "cborHex")]
    cbor_hex: String,
}

impl CardanoKeyTemplate {
    pub fn payment_signing_key(cbor_hex: String) -> Self {
        Self {
            r#type: "PaymentSigningKeyShelley_ed25519".to_string(),
            description: "Payment Signing Key".to_string(),
            cbor_hex,
        }
    }

    pub fn payment_verification_key(cbor_hex: String) -> Self {
        Self {
            r#type: "PaymentVerificationKeyShelley_ed25519".to_string(),
            description: "Payment Verification Key".to_string(),
            cbor_hex,
        }
    }

    pub fn stake_signing_key(cbor_hex: String) -> Self {
        Self {
            r#type: "StakeSigningKeyShelley_ed25519".to_string(),
            description: "Stake Signing Key".to_string(),
            cbor_hex,
        }
    }

    pub fn stake_verification_key(cbor_hex: String) -> Self {
        Self {
            r#type: "StakeVerificationKeyShelley_ed25519".to_string(),
            description: "Stake Verification Key".to_string(),
            cbor_hex,
        }
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let content = serde_json::to_string(&self).map_err(|e| Error::Json(e.to_string()))?;
        write_content(&content, path).map_err(|e| Error::Io(e.to_string()))
    }
}
