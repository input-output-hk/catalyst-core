#[derive(PartialEq, Eq, Debug)]
pub enum ApplicationExitCode {
    WriteSettingsError = 10,
    LoadSettingsError,
    DBConnectionError,
    ServiceVersionError,
}

impl ApplicationExitCode {
    // TODO: this method can be generalize once std::num new features is stabilized.
    // https://doc.rust-lang.org/0.12.0/std/num/trait.Num.html
    // https://doc.rust-lang.org/0.12.0/std/num/trait.FromPrimitive.html
    pub fn from_i32(n: i32) -> Option<Self> {
        match n {
            10 => Some(Self::WriteSettingsError),
            11 => Some(Self::LoadSettingsError),
            12 => Some(Self::DBConnectionError),
            13 => Some(Self::ServiceVersionError),
            _ => None,
        }
    }
}

impl Into<i32> for ApplicationExitCode {
    fn into(self) -> i32 {
        self as i32
    }
}
