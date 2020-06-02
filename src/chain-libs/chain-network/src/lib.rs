#![warn(clippy::all)]

pub mod core;
pub mod data;
pub mod error;
pub mod grpc;

/// Version of the protocol implemented by this crate.
///
/// Note that until the protocol is stabilized, breaking changes may still
/// occur without changing this version number.
pub const PROTOCOL_VERSION: u32 = 1;
