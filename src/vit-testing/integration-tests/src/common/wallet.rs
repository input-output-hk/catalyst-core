use std::path::Path;
use vitup::scenario::wallet::WalletProxyController;

pub fn iapyx_from_secret_key<P: AsRef<Path>>(
    secret: P,
    proxy: &WalletProxyController,
) -> Result<iapyx::Controller, iapyx::ControllerBuilderError> {
    iapyx::ControllerBuilder::default()
        .from_client(proxy.client())?
        .from_secret_file(secret.as_ref())?
        .build()
}

pub fn iapyx_from_qr<P: AsRef<Path>>(
    qr: P,
    pin: &str,
    proxy: &WalletProxyController,
) -> Result<iapyx::Controller, iapyx::ControllerBuilderError> {
    iapyx::ControllerBuilder::default()
        .from_client(proxy.client())?
        .from_qr(qr.as_ref(), pin)?
        .build()
}
