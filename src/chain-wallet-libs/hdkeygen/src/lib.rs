#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod account;
pub mod bip44;
mod key;
pub mod rindex;

pub use self::key::{Key, KeyRange};
