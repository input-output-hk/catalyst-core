pub mod csv;
pub mod serde;

use ::serde::Deserialize;
use rust_decimal::Decimal;
use std::{fs::File, path::Path};

const DEFAULT_DECIMAL_PRECISION: u32 = 10;

#[track_caller]
pub fn assert_are_close(a: Decimal, b: Decimal) {
    assert_eq!(
        a.round_dp(DEFAULT_DECIMAL_PRECISION),
        b.round_dp(DEFAULT_DECIMAL_PRECISION)
    );
}

pub fn json_from_file<T: for<'a> Deserialize<'a>>(path: impl AsRef<Path>) -> color_eyre::Result<T> {
    Ok(serde_json::from_reader(File::open(path)?)?)
}
