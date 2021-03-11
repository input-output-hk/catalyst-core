#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum NetworkType {
    #[serde(rename = "mainnet")]
    Mainnet,
    #[serde(rename = "testnet")]
    Testnet(u32),
}
