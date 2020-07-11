//! BIP44 addressing
//!
//! provides all the logic to create safe sequential addresses
//! using BIP44 specification.
//!

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod bip44;
mod derivation;
mod derivation_path;
pub mod rindex;

pub use self::{
    derivation::{
        Derivation, DerivationError, DerivationRange, HardDerivation, HardDerivationRange,
        ParseDerivationError, SoftDerivation, SoftDerivationRange,
    },
    derivation_path::{AnyScheme, DerivationPath, DerivationPathRange, ParseDerivationPathError},
};
