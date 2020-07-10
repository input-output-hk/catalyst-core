use crate::prelude::file_exists_and_not_empty;
use assert_fs::{
    assert::PathAssert,
    fixture::{ChildPath, PathChild},
    TempDir,
};
use bawawa::{Command, Error, Process, Program, StandardError, StandardOutput};
use futures::stream::Stream;
use std::path::PathBuf;
use tokio_codec::LinesCodec;
pub struct Openssl {
    program: Program,
}

impl Openssl {
    pub fn new() -> Result<Self, Error> {
        Ok(Openssl {
            program: Program::new("openssl".to_owned())?,
        })
    }

    pub fn version(&self) -> Result<String, Error> {
        let mut openssl = Command::new(self.program.clone());
        openssl.argument("version");
        self.echo_stdout(openssl)
    }

    fn echo_stdout(&self, cmd: Command) -> Result<String, Error> {
        let captured = Process::spawn(cmd.clone())?
            .capture_stdout(LinesCodec::new())
            .capture_stderr(LinesCodec::new())
            .wait();
        println!("{}", cmd);

        let lines: Vec<String> = captured
            .map(|r| r.unwrap_or_else(|_| "".to_owned()))
            .collect();
        Ok(lines.join("\n"))
    }

    pub fn genrsa(&self, length: u32, out_file: &ChildPath) -> Result<String, Error> {
        let mut openssl = Command::new(self.program.clone());
        openssl.arguments(&[
            "genrsa",
            "-out",
            &path_to_str(out_file),
            &length.to_string(),
        ]);
        self.echo_stdout(openssl)
    }

    pub fn pkcs8(&self, in_file: &ChildPath, out_file: &ChildPath) -> Result<String, Error> {
        let mut openssl = Command::new(self.program.clone());
        openssl.arguments(&[
            "pkcs8",
            "-topk8",
            "-inform",
            "PEM",
            "-outform",
            "PEM",
            "-in",
            &path_to_str(in_file),
            "-out",
            &path_to_str(out_file),
            "-nocrypt",
        ]);
        self.echo_stdout(openssl)
    }

    pub fn req(&self, prv_key: &ChildPath, out_cert: &ChildPath) -> Result<String, Error> {
        let mut openssl = Command::new(self.program.clone());
        openssl.arguments(&[
            "req",
            "-new",
            "-nodes",
            "-key",
            &path_to_str(prv_key),
            "-out",
            &path_to_str(out_cert),
            "-batch",
        ]);
        self.echo_stdout(openssl)
    }

    pub fn x509(
        &self,
        prv_key: &ChildPath,
        in_cert: &ChildPath,
        out_cert: &ChildPath,
    ) -> Result<String, Error> {
        let mut openssl = Command::new(self.program.clone());
        openssl.arguments(&[
            "x509",
            "-req",
            "-days",
            &3650.to_string(),
            "-in",
            &path_to_str(in_cert),
            "-signkey",
            &path_to_str(prv_key),
            "-out",
            &path_to_str(out_cert),
        ]);
        self.echo_stdout(openssl)
    }

    pub fn convert_to_der(
        &self,
        in_cert: &ChildPath,
        out_der: &ChildPath,
    ) -> Result<String, Error> {
        let mut openssl = Command::new(self.program.clone());
        openssl.arguments(&[
            "x509",
            "-inform",
            "pem",
            "-in",
            &path_to_str(in_cert),
            "-outform",
            "der",
            "-out",
            &path_to_str(out_der),
        ]);
        self.echo_stdout(openssl)
    }
}

fn path_to_str(path: &ChildPath) -> String {
    let path_buf: PathBuf = path.path().into();
    path_buf.as_os_str().to_owned().into_string().unwrap()
}

pub fn generate_keys(temp_dir: &TempDir) -> (PathBuf, PathBuf) {
    let openssl = Openssl::new().expect("no openssla installed.");
    let prv_key_file = temp_dir.child("prv.key");
    let pk8_key_file = temp_dir.child("prv.pk8");
    let csr_cert_file = temp_dir.child("cert.csr");
    let cert_file = temp_dir.child("cert.crt");
    let der_file = temp_dir.child("cert.der");

    openssl
        .genrsa(2048, &prv_key_file)
        .expect("cannot generate private key.");
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

    (prv_key_file.path().into(), cert_file.path().into())
}
