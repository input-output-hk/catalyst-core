use std::io;
use std::path::Path;
use std::str::FromStr;

#[derive(Clone)]
pub struct GenesisBlock {
    pub block0_path: String,
    pub block0: Vec<u8>,
}

impl GenesisBlock {
    pub(crate) fn is_fund_id(&self, fund_id: i32) -> bool {
        Path::new(&self.block0_path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            == format!("fund{}", fund_id)
    }
}

impl FromStr for GenesisBlock {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            block0: std::fs::read(s).unwrap_or_default(),
            block0_path: s.to_string(),
        })
    }
}
