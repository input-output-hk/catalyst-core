use super::Error;
use std::fs::{self, File};
use std::path::Path;

pub fn load_cert(filename: &Path) -> Result<Vec<rustls::Certificate>, Error> {
    let certfile = fs::File::open(filename)?;
    let mut reader = std::io::BufReader::new(certfile);

    match rustls_pemfile::read_one(&mut reader)? {
        Some(rustls_pemfile::Item::X509Certificate(cert)) => Ok(vec![rustls::Certificate(cert)]),
        Some(_) => {
            // TODO: a more specific error could be useful (ExpectedCertFoundKey?)
            Err(Error::InvalidCertificate)
        }
        // not a pemfile
        None | Some(_) => Err(Error::InvalidCertificate),
    }
}

pub fn load_private_key(filename: &Path) -> Result<rustls::PrivateKey, Error> {
    let keyfile = File::open(filename)?;
    let mut reader = std::io::BufReader::new(keyfile);

    match rustls_pemfile::read_one(&mut reader)? {
        Some(rustls_pemfile::Item::RSAKey(key)) => Ok(rustls::PrivateKey(key)),
        Some(rustls_pemfile::Item::PKCS8Key(key)) => Ok(rustls::PrivateKey(key)),
        None | Some(_) => Err(Error::InvalidKey),
    }
}
