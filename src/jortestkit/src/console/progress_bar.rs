use std::{fmt, convert::Infallible};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProgressBarMode {
    Monitor,
    Standard,
    None,
}

pub fn parse_progress_bar_mode_from_str(progress_bar_mode: &str) -> Result<ProgressBarMode, Infallible> {
    let progress_bar_mode_lowercase: &str = &progress_bar_mode.to_lowercase();
    let result = match progress_bar_mode_lowercase {
        "standard" => ProgressBarMode::Standard,
        "none" => ProgressBarMode::None,
        _ => ProgressBarMode::Monitor,
    };

    Ok(result)
}

impl fmt::Display for ProgressBarMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
