use serde_yaml::Error as SerdeError;
use std::{collections::BTreeMap, process::Output};

pub trait ProcessOutput {
    fn as_lossy_string(&self) -> String;
    fn as_single_line(&self) -> String;
    fn as_multi_line(&self) -> Vec<String>;
    fn as_multi_node_yaml(&self) -> Vec<BTreeMap<String, String>>;
    fn as_single_node_yaml(&self) -> BTreeMap<String, String>;
    fn try_as_single_node_yaml(&self) -> Result<BTreeMap<String, String>, SerdeError>;
    fn err_as_lossy_string(&self) -> String;
    fn err_as_single_line(&self) -> String;
}

impl ProcessOutput for Output {
    fn as_lossy_string(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    fn as_multi_line(&self) -> Vec<String> {
        let content = self.as_lossy_string();
        content
            .split('\n')
            .filter(|x| !x.trim().is_empty())
            .map(|x| x.to_string())
            .collect()
    }

    fn as_single_line(&self) -> String {
        let mut content = self.as_lossy_string();
        if content.ends_with('\n') {
            let len = content.len();
            content.truncate(len - 1);
        }
        content.trim().to_string()
    }

    fn err_as_lossy_string(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }

    fn err_as_single_line(&self) -> String {
        let mut content = self.err_as_lossy_string();
        if content.ends_with('\n') {
            let len = content.len();
            content.truncate(len - 1);
        }
        content
    }

    fn as_multi_node_yaml(&self) -> Vec<BTreeMap<String, String>> {
        let content = self.as_lossy_string();
        let deserialized_map: Vec<BTreeMap<String, String>> =
            serde_yaml::from_str(&content).unwrap();
        deserialized_map
    }

    fn as_single_node_yaml(&self) -> BTreeMap<String, String> {
        let content = self.as_lossy_string();
        let deserialized_map: BTreeMap<String, String> = serde_yaml::from_str(&content).unwrap();
        deserialized_map
    }

    fn try_as_single_node_yaml(&self) -> Result<BTreeMap<String, String>, SerdeError> {
        let content = self.as_lossy_string();
        serde_yaml::from_str(&content)
    }
}
