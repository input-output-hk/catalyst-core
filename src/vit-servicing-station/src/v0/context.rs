use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type ChainData = Value;
pub type ChainDataStore = HashMap<String, ChainData>;

pub type SharedContext = Arc<RwLock<Context>>;

#[derive(Clone)]
pub struct Context {
    pub static_chain_data: ChainDataStore,
}

impl Context {
    pub fn new(static_chain_data: ChainDataStore) -> Self {
        Self { static_chain_data }
    }
}

pub fn new_default_context() -> SharedContext {
    new_shared_context(Path::new("./resources/v0/chain_data.json"))
}

pub fn new_shared_context(file_path: &Path) -> SharedContext {
    let chain_data = match load_file_data(file_path) {
        Ok(data) => data,
        Err(err) => panic!("Error reading chain data file: {}", err),
    };
    let static_chain_data: ChainDataStore = match serde_json::from_str(&chain_data) {
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

#[cfg(test)]
mod test {
    use crate::v0::context::new_default_context;

    #[test]
    fn load_default() {
        new_default_context();
    }
}
