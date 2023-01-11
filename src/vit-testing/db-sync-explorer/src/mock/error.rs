use cardano_serialization_lib::error::JsError;
use mainnet_lib::CatalystBlockFrostApiError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("internal error: {0}")]
    Internal(String),
    #[error(transparent)]
    Js(#[from] JsError),
    #[error("cannot parse voting key due to: {0}")]
    PublicKeyFromStr(String),
    #[error(transparent)]
    CatalystBlockFrost(#[from] CatalystBlockFrostApiError),
}
