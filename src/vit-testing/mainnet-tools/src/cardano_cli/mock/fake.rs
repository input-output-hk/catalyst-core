use core::fmt::Formatter;
use prettytable::{format, Table};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;

/// Arbitrary blockchain tip
#[must_use]
pub fn tip() -> Tip {
    Tip::default()
}

/// Arbitraty query utxo response
#[must_use]
pub fn utxo() -> QueryUTxO {
    QueryUTxO::default()
}

/// Arbitrary hash
#[must_use]
pub fn hash() -> String {
    "a3a93043d015e9bb089b1a90d59b1922dffb9684b5c64a61426b6134e348123d".to_string()
}

/// Arbitrary address
#[must_use]
pub fn address() -> String {
    "addr1q9e0wsxghc395p8e3ff0zx2gdzurveq0pcu68lyq2z59vc5squl6qr0re0pe2x5syq7j5qf77q6s7zl43nass8u85vgscexsev".to_string()
}

/// Arbitrary stake address
#[must_use]
pub fn stake_address() -> String {
    "stake1uxgqw0aqph3uhsu4r2gzq0f2qyl0qdg0p06ce7cgr7r6xygwmtwku".to_string()
}

/// Arbitrary Response from Cardano CLI on submit transaction command
#[must_use]
pub fn submit() -> String {
    "Transaction successfully submitted.".to_string()
}

/// Response from transaction sign command
#[must_use]
pub fn sign() -> String {
    json!({
        "type": "TxSignedShelley",
        "description": "",
        "cborHex": "83a500828258205761bdc4fd016ee0d52ac759ae6c0e8e0943d4892474283866a07f9768e48fee00825820e6701be50c87d8d584985edd4cf39799e1445bd37907027c44d08c7da79ea23200018182583900fec5a902e307707b6ab3de38104918c0e33cf4c3408e6fcea4f0a199c13582aec9a44fcc6d984be003c5058c660e1d2ff1370fd8b49ba73f1b00001e0369444cd7021a0002c329031a00ce0fc70758202386abf617780a925495f38f23d7bc594920ff374f03f3d7517a4345e355b047a1008182582099d1d0c4cdc8a4b206066e9606c6c3729678bd7338a8eab9bffdffa39d3df9585840af346c11fe7a222008f5b1b50fbc23a0cbc3d783bf4461f21353e8b5eb664adadb34291197e039e467d2a68346921879d1212bd0d54245a9e110162ecae9190ba219ef64a201582071ce673ef64b4ac1fb758b65df01b036665d4498256335e93e28b869568d9ed80258209be513df12b3fabe7c1b8c3f9fab0968eb2168d5689bf981c2f7c35b11718b2719ef65a101584057267d94e5bae64fa236924b83ce7411fef10bd5d73aca7af8403053cf2dc2e3621f7d253bf90933e2bc0bfb56146cf0a13925d9f96d6d06b0b798bc41d4000d"
    }).to_string()
}

/// Helper struct which imitates response from tip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tip {
    /// tip hash
    pub hash: String,
    /// tip block height
    pub block: u64,
    /// tip slot
    pub slot: u64,
    /// tip sync progress
    pub sync_progress: String,
    /// tip era
    pub era: String,
    /// tip epoch
    pub epoch: u64,
}

impl Default for Tip {
    fn default() -> Self {
        Self {
            hash: hash(),
            block: 6_589_745,
            slot: 47_163_888,
            sync_progress: "100.00".to_string(),
            era: "Alonzo".to_string(),
            epoch: 306,
        }
    }
}

/// Helper struct which imitates response from utxo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTxO {
    /// Utxo tx hash
    pub tx_hash: String,
    /// Utxo tx id
    pub tx_ix: u64,
    /// Utxo ada amount
    pub amount: u64,
}

/// Helper struct which imitates response from query utxo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryUTxO {
    utxos: Vec<UTxO>,
}

impl fmt::Display for QueryUTxO {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();
        let format = format::FormatBuilder::new()
            .column_separator('|')
            .separators(
                &[format::LinePosition::Top, format::LinePosition::Bottom],
                format::LineSeparator::new('-', ' ', ' ', ' '),
            )
            .padding(1, 1)
            .build();
        table.set_format(format);
        table.set_titles(row!["TxHash", "TxIx", "Amount"]);

        for utxo in &self.utxos {
            table.add_row(row![
                utxo.tx_hash.to_string(),
                utxo.tx_ix,
                format!("{} lovelace", utxo.amount)
            ]);
        }
        write!(f, "{table}")
    }
}

impl Default for QueryUTxO {
    fn default() -> Self {
        QueryUTxO {
            utxos: vec![
                UTxO {
                    tx_hash: "61d47e568b1502064906e977aae848c7aec9a76f97e7d11ad5d752e95c438011"
                        .to_string(),
                    tx_ix: 0,
                    amount: 1_379_280,
                },
                UTxO {
                    tx_hash: "ac1d8802a4e100d90ce59fb4e4573f1c7884a65197ff39810a88eb0b07de3aa6"
                        .to_string(),
                    tx_ix: 0,
                    amount: 30_000_000,
                },
                UTxO {
                    tx_hash: "69818d49963ffafe8a287ec270d05ba89493de33ddf7b5b9bcb07e97802a0f28"
                        .to_string(),
                    tx_ix: 0,
                    amount: 5_573_009,
                },
                UTxO {
                    tx_hash: "fba1526c49684722199b102bffd5b4a66ea1d490605532753fa24e12af925722"
                        .to_string(),
                    tx_ix: 0,
                    amount: 5_000_000,
                },
            ],
        }
    }
}

/// Arbitrary protocol parameters
#[must_use]
pub fn protocol_parameters() -> Value {
    json!(
    {
        "txFeePerByte": 44,
        "minUTxOValue": 34_482,
        "decentralization": 0,
        "utxoCostPerWord":  34_482,
        "stakePoolDeposit": 500_000_000,
        "poolRetireMaxEpoch": 18,
        "extraPraosEntropy": null,
        "collateralPercentage": 150,
        "stakePoolTargetNum": 500,
        "maxBlockBodySize": 73_728,
        "minPoolCost": 340_000_000,
        "maxTxSize": 16_384,
        "treasuryCut": 0.2,
        "maxBlockExecutionUnits": {
            "memory": 50_000_000,
            "steps":  4_000_000
        },
        "maxCollateralInputs": 3,
        "maxValueSize": 5_000,
        "maxBlockHeaderSize": 1_100,
        "maxTxExecutionUnits": {
            "memory": 11_250_000,
            "steps": 1_000_000
        },
        "costModels": {},
        "protocolVersion": {
            "minor": 0,
            "major": 6
        },
        "txFeeFixed": 155_381,
        "stakeAddressDeposit": 2_000_000,
        "monetaryExpansion": 0.003,
        "poolPledgeInfluence": 0.3,
        "executionUnitPrices": {
            "priceSteps": 0.000_072_1,
            "priceMemory": 0.057_7
        }
    })
}

/// Arbitrary stake certificate
#[must_use]
#[allow(dead_code)]
pub fn stake_certificate() -> String {
    "type: CertificateShelley \
    description: Stake Registration Certificate \
    cborHex: \
    18b58a03582062d632e7ee8a83769bc108e3e42a674d8cb242d7375fc2d97db9b4dd6eded6fd5820 \
    48aa7b2c8deb8f6d2318e3bf3df885e22d5d63788153e7f4040c33ecae15d3e61b0000005d21dba0 \
    001b000000012a05f200d81e820001820058203a4e813b6340dc790f772b3d433ce1c371d5c5f5de \
    46f1a68bdf8113f50e779d8158203a4e813b6340dc790f772b3d433ce1c371d5c5f5de46f1a68bdf \
    8113f50e779d80f6"
        .to_string()
}

/// Arbitrary transaction
#[must_use]
pub fn transaction() -> String {
    "[{0: [[h'C2642218EF9C5B2BC1EF66BF27C37640C0DFD159A0274C8100C852CA0B03D484', 1]], 1: [[h'01B2344AB02D3DB07A62D640E2BA6F307CB02F3BB8196E81FE68BE8B780C39E651F9564FEFB52309B211E6F5A96BE9EE2A773AEC691219789E', 0]], 2: 0, 3: 25000000}, null]".to_string()
}
