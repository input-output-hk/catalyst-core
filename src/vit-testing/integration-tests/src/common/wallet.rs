use crate::common::CardanoWallet;
use std::path::Path;
use vitup::mode::standard::WalletProxyController;

pub fn iapyx_from_mainnet(
    wallet: &CardanoWallet,
    proxy: &WalletProxyController,
) -> Result<iapyx::Controller, iapyx::ControllerBuilderError> {
    iapyx::ControllerBuilder::default()
        .with_backend_from_client(proxy.client())?
        .with_wallet_from_secret_key(wallet.catalyst_secret_key())?
        .build()
}

pub fn iapyx_from_secret_key<P: AsRef<Path>>(
    secret: P,
    proxy: &WalletProxyController,
) -> Result<iapyx::Controller, iapyx::ControllerBuilderError> {
    iapyx::ControllerBuilder::default()
        .with_backend_from_client(proxy.client())?
        .with_wallet_from_secret_file(secret.as_ref())?
        .build()
}

pub fn iapyx_from_qr<P: AsRef<Path>>(
    qr: P,
    pin: &str,
    proxy: &WalletProxyController,
) -> Result<iapyx::Controller, iapyx::ControllerBuilderError> {
    iapyx::ControllerBuilder::default()
        .with_backend_from_client(proxy.client())?
        .with_wallet_from_qr_file(qr.as_ref(), pin)?
        .build()
}
