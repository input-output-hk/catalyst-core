use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type ChainData = HashMap<String, Value>;

pub type SharedContext = Arc<RwLock<Context>>;

#[derive(Clone)]
pub struct Context {
    pub static_chain_data: ChainData,
}

impl Context {
    pub fn new(static_chain_data: ChainData) -> Self {
        Self { static_chain_data }
    }
}

pub fn new_default_context() -> SharedContext {
    let chain_data = match load_file_data(Path::new("../../../static/v0/chain_data.json")) {
        Ok(data) => data,
        Err(err) => panic!("Error reading chain data file: {}", err),
    };
    let static_chain_data: ChainData = match serde_json::from_str(&chain_data) {
        Ok(data) => data,
        Err(err) => panic!("Error parsing chain data file: {}", err),
    };
    let context = Context::new(static_chain_data);
    Arc::new(RwLock::new(context))
}

fn load_file_data(file_path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
