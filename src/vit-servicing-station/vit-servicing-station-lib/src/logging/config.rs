pub use simplelog::LevelFilter;
use simplelog::*;
use std::fs::File;

pub fn config_log(
    level: LevelFilter,
    file_log_path: Option<String>,
    config: Option<Config>,
) -> std::io::Result<()> {
    let terminal_logger = TermLogger::new(
        level,
        config.clone().unwrap_or_default(),
        TerminalMode::Mixed,
    );

    let mut log_vec: Vec<Box<dyn SharedLogger>> = vec![terminal_logger];

    if let Some(file_path) = file_log_path {
        let file_logger =
            WriteLogger::new(level, config.unwrap_or_default(), File::create(file_path)?);
        log_vec.push(file_logger);
    }

    CombinedLogger::init(log_vec)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}
