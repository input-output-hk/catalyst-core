use serde_json::Value;
use std::collections::HashMap;
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
    pub db_url: String,
}

impl Context {
    pub fn new(static_chain_data: ChainDataStore, db_url: &str) -> Self {
        Self {
            static_chain_data,
            db_url: db_url.to_string(),
        }
    }
}

pub fn new_default_context() -> SharedContext {
    new_shared_context(
        Path::new("./resources/v0/chain_data.json"),
        "./db/database.sqlite3",
    )
}

pub fn new_shared_context(file_path: &Path, db_url: &str) -> SharedContext {
    let chain_data = match load_file_data(file_path) {
        Ok(data) => data,
        Err(err) => panic!("Error reading chain data file: {}", err),
    };
    let static_chain_data: ChainDataStore = match serde_json::from_str(&chain_data) {
        Ok(data) => data,
        Err(err) => panic!("Error parsing chain data file: {}", err),
    };
    let context = Context::new(static_chain_data, db_url);
    Arc::new(RwLock::new(context))
}

fn load_file_data(file_path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::v0::context::{new_default_context, SharedContext};

    /// Build a fake context, returns the id, the fake data and the context object to check
    /// Returns `(id, data, context)`
    pub fn fake_data_context() -> (String, ChainData, SharedContext) {
        // build fake data
        let data = r#"{"foo" : "bar"}"#;
        let json_data: ChainData = serde_json::from_str(data).unwrap();

        let id = String::from("foo");

        // build fake context chain data
        let mut context_data = ChainDataStore::new();
        context_data.insert(id.clone(), json_data.clone());

        // Empty ("") db_url should create a temporary file db for sqlite3
        let context = Arc::new(RwLock::new(Context::new(context_data, "")));
        (id.clone(), json_data.clone(), context)
    }

    #[test]
    fn load_default() {
        new_default_context();
    }
}
