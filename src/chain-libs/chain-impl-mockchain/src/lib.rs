#![warn(clippy::all)]

#[cfg(any(test, feature = "property-test-api"))]
#[macro_use]
extern crate quickcheck;

pub mod account;
pub mod accounting;
pub mod block;
pub mod certificate;
pub mod chaineval;
pub mod chaintypes;
pub mod config;
mod date;
pub mod error;
pub mod fee;
pub mod fragment;
pub mod header;
pub mod key;
pub mod leadership;
pub mod ledger;
pub mod legacy;
pub mod milli;
pub mod multisig;
pub mod multiverse;
pub mod rewards;
pub mod setting;
pub mod stake;
pub mod transaction;
pub mod treasury;
pub mod update;
pub mod utxo;
pub mod value;
pub mod vote;

#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(test)] {
        pub mod testing;
        extern crate ed25519_bip32;
    } else if #[cfg(feature = "property-test-api")] {
        pub mod testing;
        extern crate ed25519_bip32;
    }
}
