pub mod csv;
pub mod serde;

use rust_decimal::Decimal;

const DEFAULT_DECIMAL_PRECISION: u32 = 10;

#[track_caller]
pub fn assert_are_close(a: Decimal, b: Decimal) {
    assert_eq!(
        a.round_dp(DEFAULT_DECIMAL_PRECISION),
        b.round_dp(DEFAULT_DECIMAL_PRECISION)
    );
}
