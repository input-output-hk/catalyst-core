use std::io::Write;
use std::path::Path;
use vitup::mode::mock::Configuration;

#[cfg(feature = "soak")]
mod soak;
mod startup;

pub fn write_config<P: AsRef<Path>>(config: &Configuration, output: P) {
    let content = serde_json::to_string(&config).unwrap();
    let mut file = std::fs::File::create(&output).unwrap();
    file.write_all(content.as_bytes()).unwrap()
}
