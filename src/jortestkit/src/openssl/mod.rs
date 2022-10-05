use crate::prelude::file_exists_and_not_empty;
use assert_fs::{
    assert::PathAssert,
    fixture::{ChildPath, PathChild},
    TempDir,
};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
type Error = std::io::Error;
use crate::process::output_extensions::ProcessOutput;

pub struct Openssl {
    program: PathBuf,
}

impl Openssl {
    pub fn new() -> Result<Self, Error> {
        Ok(Openssl {
            program: PathBuf::from_str("openssl").unwrap(),
        })
    }

    pub fn version(&self) -> Result<String, Error> {
        Ok(Command::new(self.program.clone())
            .arg("version")
            .output()?
            .as_lossy_string())
    }

    pub fn genrsa(&self, length: u32, out_file: &ChildPath) -> Result<String, Error> {
        Ok(Command::new(self.program.clone())
            .arg("genrsa")
            .arg("-out")
            .arg(path_to_str(out_file))
            .arg(length.to_string())
            .output()?
            .as_lossy_string())
    }

    pub fn pkcs8(&self, in_file: &ChildPath, out_file: &ChildPath) -> Result<String, Error> {
        Ok(Command::new(self.program.clone())
            .arg("pkcs8")
            .arg("-topk8")
            .arg("-inform")
            .arg("PEM")
            .arg("-outform")
            .arg("PEM")
            .arg("-in")
            .arg(path_to_str(in_file))
            .arg("-out")
            .arg(path_to_str(out_file))
            .arg("-nocrypt")
            .output()?
            .as_lossy_string())
    }

    pub fn req(&self, prv_key: &ChildPath, out_cert: &ChildPath) -> Result<String, Error> {
        Ok(Command::new(self.program.clone())
            .arg("req")
            .arg("-new")
            .arg("-nodes")
            .arg("-key")
            .arg(path_to_str(prv_key))
            .arg("-out")
            .arg(path_to_str(out_cert))
            .arg("-batch")
            .output()?
            .as_lossy_string())
    }

    pub fn x509(
        &self,
        prv_key: &ChildPath,
        in_cert: &ChildPath,
        out_cert: &ChildPath,
    ) -> Result<String, Error> {
        Ok(Command::new(self.program.clone())
            .arg("x509")
            .arg("-req")
            .arg("-days")
            .arg(3650.to_string())
            .arg("-in")
            .arg(path_to_str(in_cert))
            .arg("-signkey")
            .arg(path_to_str(prv_key))
            .arg("-out")
            .arg(path_to_str(out_cert))
            .output()?
            .as_lossy_string())
    }

    pub fn convert_to_der(
        &self,
        in_cert: &ChildPath,
        out_der: &ChildPath,
    ) -> Result<String, Error> {
        Ok(Command::new(self.program.clone())
            .arg("x509")
            .arg("-inform")
            .arg("pem")
            .arg("-in")
            .arg(path_to_str(in_cert))
            .arg("-outform")
            .arg("der")
            .arg("-out")
            .arg(path_to_str(out_der))
            .output()?
            .as_lossy_string())
    }
}

fn path_to_str(path: &ChildPath) -> String {
    let path_buf: PathBuf = path.path().into();
    path_buf.as_os_str().to_owned().into_string().unwrap()
}

pub fn generate_keys(temp_dir: &TempDir) -> (PathBuf, PathBuf, PathBuf) {
    let openssl = Openssl::new().expect("no openssla installed.");
    let prv_key_file = temp_dir.child("prv.key");
    let pk8_key_file = temp_dir.child("prv.pk8");
    let csr_cert_file = temp_dir.child("cert.csr");
    let cert_file = temp_dir.child("cert.crt");
    let der_file = temp_dir.child("cert.der");

    println!(
        "{}",
        openssl
            .genrsa(2048, &prv_key_file)
            .expect("cannot generate private key.")
    );
    openssl
        .pkcs8(&prv_key_file, &pk8_key_file)
        .expect("cannot wrap private key in PKC8");
    openssl
        .req(&prv_key_file, &csr_cert_file)
        .expect("cannot register a self-signed certificate for private key");
    openssl
        .x509(&prv_key_file, &csr_cert_file, &cert_file)
        .expect("cannot generate a self-signed certificate for private key");
    openssl
        .convert_to_der(&cert_file, &der_file)
        .expect("cannot convert cert file to der file");

    prv_key_file.assert(file_exists_and_not_empty());
    cert_file.assert(file_exists_and_not_empty());

    (
        prv_key_file.path().into(),
        cert_file.path().into(),
        der_file.path().into(),
    )
}
