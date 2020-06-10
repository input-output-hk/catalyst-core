// Generated protobuf/gRPC code.
mod proto {
    tonic::include_proto!("iohk.chain.node");
}

pub mod client;
pub mod server;

#[cfg(feature = "legacy")]
pub mod legacy;

mod convert;
mod streaming;

pub use client::Client;
pub use server::{NodeService, Server};
