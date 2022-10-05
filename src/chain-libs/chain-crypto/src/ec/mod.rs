//! Module defining the Group Elements and Scalar structures in one primer order group (over sec2
//! curves), or the other (ristretto255).
#[macro_use]
mod macros;
#[cfg(feature = "p256k1")]
pub mod p256k1;
pub mod ristretto255;
