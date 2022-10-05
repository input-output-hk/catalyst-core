//!
//! Cardano Legacy Address generation and parsing
//!
#[macro_use]
extern crate cbor_event;

extern crate cryptoxide;

extern crate ed25519_bip32;

mod address;
mod base58;
mod crc32;

#[cfg(not(feature = "with-bench"))]
mod cbor;
#[cfg(feature = "with-bench")]
pub mod cbor;

pub use address::{Addr, AddressMatchXPub, Attributes, ExtendedAddr, ParseExtendedAddrError};
