use jortestkit::load::Configuration;

pub struct IapyxLoadConfig {
    pub config: Configuration,
    pub measure: bool,
    pub address: String,
}

impl IapyxLoadConfig {
    pub fn new(config: Configuration, measure: bool, address: String) -> Self {
        Self {
            config,
            measure,
            address,
        }
    }
}
