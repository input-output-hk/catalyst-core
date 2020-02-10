#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(test)] {
        extern crate quickcheck;
    } else if #[cfg(feature = "property-test-api")] {
        extern crate quickcheck;
    }
}

pub use chain_ser::abor;
pub use chain_ser::mempack;
pub use chain_ser::packer;
pub mod property;
