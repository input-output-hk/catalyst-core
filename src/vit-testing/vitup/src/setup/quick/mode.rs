use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Service,
    Interactive,
    Endless,
}

pub fn parse_mode_from_str(mode: &str) -> Mode {
    let mode_lowercase: &str = &mode.to_lowercase();
    match mode_lowercase {
        "service" => Mode::Service,
        "interactive" => Mode::Interactive,
        _ => Mode::Endless,
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
