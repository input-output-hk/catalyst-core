#![allow(dead_code)]
/// This build script serves as a way to select a single crypto backend to use
/// without polluting the code with chains of if-else cfg everywhere a mutually
/// exclusive choice about the backend has to be made.
///
/// The backend is selected with the following rules:
/// * if a non-default feature is selected, activate the corresponding backend
/// * If no non-default feature is selected, activate the default backend
///
/// Flags exported by this module are guaranteed to be mutually exclusive.

const BACKEND_FLAG_P256K1: &str = "__internal_ex_backend_p256k1";
const BACKEND_FLAG_RISTRETTO255: &str = "__internal_ex_backend_ristretto255";

fn main() {
    cfg_if::cfg_if! {
        if #[cfg(feature = "p256k1")] {
            println!("cargo:rustc-cfg=crypto_backend=\"{}\"", BACKEND_FLAG_P256K1);
        } else if #[cfg(feature = "ristretto255")] {
            println!("cargo:rustc-cfg=crypto_backend=\"{}\"", BACKEND_FLAG_RISTRETTO255);
        } else {
            compile_error!("one of the features \"p256k1\", \"ristretto255\' has to be selected as the backend");
        }
    }
}
