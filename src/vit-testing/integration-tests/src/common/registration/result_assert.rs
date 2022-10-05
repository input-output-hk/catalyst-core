use catalyst_toolbox::kedqr::PinReadMode;
use iapyx::utils::qr::{Secret, SecretFromQrCode};
use registration_service::client::LegacyResultInfo;

pub trait RegistrationResultAsserts {
    fn assert_qr_equals_to_sk(&self);
}

impl RegistrationResultAsserts for LegacyResultInfo {
    fn assert_qr_equals_to_sk(&self) {
        let bech32_key = Secret::from_file(
            &self.qr_code,
            PinReadMode::FromFileName(self.qr_code.clone()),
        )
        .expect("unable to read qr code")
        .to_bech32()
        .expect("unable to export key as bech32");

        assert_eq!(
            self.leak_sk(),
            bech32_key,
            "secret key from qr is not equal to used during registration"
        )
    }
}
