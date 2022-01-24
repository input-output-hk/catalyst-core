use hersir::config::SessionMode;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Service,
    Interactive,
    Standard,
    Monitor,
}

pub fn parse_mode_from_str(mode: &str) -> Mode {
    let mode_lowercase: &str = &mode.to_lowercase();
    match mode_lowercase {
        "service" => Mode::Service,
        "interactive" => Mode::Interactive,
        "monitor" => Mode::Monitor,
        _ => Mode::Standard,
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[allow(clippy::from_over_into)]
impl Into<SessionMode> for Mode {
    fn into(self) -> SessionMode {
        match self {
            Self::Monitor => SessionMode::Monitor,
            Self::Service => SessionMode::Monitor,
            Self::Interactive => SessionMode::Interactive,
            Self::Standard => SessionMode::Standard,
        }
    }
}
