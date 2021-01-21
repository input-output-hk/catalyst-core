#[macro_use(error_chain, bail)]
extern crate error_chain;

pub mod error;
pub mod interactive;
pub mod manager;
pub mod scenario;
pub mod setup;

use error::Result;

use scenario::vit_station;
use scenario::wallet;

#[cfg(test)]
pub mod tests {
    use glob::{glob, glob_with};
    use json;
    use walkdir::WalkDir;
    #[test]
    pub fn test() {
        let mut data = json::JsonValue::new_array();
        let root = "vit_backend";
        for entry in WalkDir::new(root) {
            entry.path().display();

            let mut data = json::JsonValue::new_array();

            let entry = entry.unwrap();
            println!("{}", entry.path().display());
        }
    }
}
