use crate::config::NetworkType;
use crate::request::Request;
use crate::utils::CommandExt as _;
use jormungandr_integration_tests::common::jcli::JCli;
use jortestkit::prelude::read_file;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitStatus;
use std::str::FromStr;
use thiserror::Error;

pub struct VoteRegistrationJobBuilder {
    job: VoteRegistrationJob,
}

impl VoteRegistrationJobBuilder {
    pub fn new() -> Self {
        Self {
            job: Default::default(),
        }
    }

    pub fn with_jcli<P: AsRef<Path>>(mut self, jcli: P) -> Self {
        self.job.jcli = jcli.as_ref().to_path_buf();
        self
    }

    pub fn with_cardano_cli<P: AsRef<Path>>(mut self, cardano_cli: P) -> Self {
        self.job.cardano_cli = cardano_cli.as_ref().to_path_buf();
        self
    }

    pub fn with_kedqr<P: AsRef<Path>>(mut self, vit_kedqr: P) -> Self {
        self.job.vit_kedqr = vit_kedqr.as_ref().to_path_buf();
        self
    }

    pub fn with_voter_registration<P: AsRef<Path>>(mut self, voter_registration: P) -> Self {
        self.job.voter_registration = voter_registration.as_ref().to_path_buf();
        self
    }

    pub fn with_network(mut self, network: NetworkType) -> Self {
        self.job.network = network;
        self
    }

    pub fn with_working_dir<P: AsRef<Path>>(mut self, working_dir: P) -> Self {
        self.job.working_dir = working_dir.as_ref().to_path_buf();
        self
    }

    pub fn build(self) -> VoteRegistrationJob {
        self.job
    }
}

pub struct VoteRegistrationJob {
    jcli: PathBuf,
    cardano_cli: PathBuf,
    voter_registration: PathBuf,
    vit_kedqr: PathBuf,
    network: NetworkType,
    working_dir: PathBuf,
}

impl Default for VoteRegistrationJob {
    fn default() -> Self {
        Self {
            jcli: PathBuf::from_str("jcli").unwrap(),
            cardano_cli: PathBuf::from_str("cardano-cli").unwrap(),
            voter_registration: PathBuf::from_str("voter-registration").unwrap(),
            vit_kedqr: PathBuf::from_str("vit-kedqr").unwrap(),
            network: NetworkType::Mainnet,
            working_dir: PathBuf::from_str(".").unwrap(),
        }
    }
}

impl VoteRegistrationJob {
    pub fn generate_payment_address<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        verification_key: P,
        output: Q,
    ) -> Result<ExitStatus, Error> {
        let mut command = Command::new(&self.cardano_cli);
        command
            .arg("address")
            .arg("build")
            .arg("--verification-key-file")
            .arg(verification_key.as_ref())
            .arg("--out-file")
            .arg(output.as_ref())
            .arg_network(self.network);
        println!("generate payment addres: {:?}", command);
        command.status().map_err(Into::into)
    }

    pub fn start(&self, request: Request) -> Result<(), Error> {
        println!("saving payment.skey...");
        let payment_skey = CardanoKeyTemplate::payment_signing_key(request.payment_skey);
        let payment_skey_path = Path::new(&self.working_dir).join("payment.skey");
        payment_skey.write_to_file(&payment_skey_path)?;
        println!("payment.skey saved");

        println!("saving payment.vkey...");
        let payment_vkey = CardanoKeyTemplate::payment_verification_key(request.payment_vkey);
        let payment_vkey_path = Path::new(&self.working_dir).join("payment.vkey");
        payment_vkey.write_to_file(&payment_vkey_path)?;
        println!("payment.vkey saved");

        println!("saving stake.skey...");
        let stake_skey = CardanoKeyTemplate::stake_signing_key(request.stake_skey);
        let stake_skey_path = Path::new(&self.working_dir).join("stake.skey");
        stake_skey.write_to_file(stake_skey_path)?;
        println!("stake.skey saved");

        println!("saving stake.vkey...");
        let stake_vkey = CardanoKeyTemplate::stake_verification_key(request.stake_vkey);
        let stake_vkey_path = Path::new(&self.working_dir).join("stake.vkey");
        stake_vkey.write_to_file(&stake_vkey_path)?;
        println!("stake.vkey saved");

        println!("saving catalyst-vote.skey...");
        let jcli = JCli::new(self.jcli.clone());
        let private_key = jcli.key().generate_default();
        let private_key_path = Path::new(&self.working_dir).join("catalyst-vote.skey");
        write_content(&private_key, &private_key_path)?;
        println!("catalyst-vote.skey saved");

        println!("saving catalyst-vote.pkey...");
        let public_key = jcli.key().convert_to_public_string(&private_key);
        let public_key_path = Path::new(&self.working_dir).join("catalyst-vote.pkey");
        write_content(&public_key, &public_key_path)?;
        println!("catalyst-vote.pkey saved");

        println!("saving payment.addr...");
        let payment_address_path = Path::new(&self.working_dir).join("payment.addr");
        self.generate_payment_address(&payment_vkey_path, &payment_address_path)?;
        println!("payment.addr saved");

        let payment_address = read_file(&payment_address_path);
        let vote_registration_path = Path::new(&self.working_dir).join("vote-registration.tx");

        let mut command = Command::new(&self.voter_registration);
        command
            .arg("--payment-signing-key")
            .arg(&payment_skey_path)
            .arg("--payment-address")
            .arg(&payment_address)
            .arg("--stake-signing-key")
            .arg(&payment_skey_path)
            .arg("--vote-public-key")
            .arg(&public_key_path)
            .arg_network(self.network)
            .arg("--mary-era")
            .arg("--cardano-mode")
            .arg("--sign")
            .arg("--out-file")
            .arg(&vote_registration_path);

        println!("Running voter-registration: {:?}", command);
        command.status()?;
        println!("voter-registration finished");

        let mut command = Command::new(&self.cardano_cli);
        command
            .arg("transaction")
            .arg("submit")
            .arg("--cardano-mode")
            .arg_network(self.network)
            .arg("--tx-file")
            .arg(&vote_registration_path);

        println!("Running cardano_cli: {:?}", command);
        command.status()?;
        println!("cardano_cli finished");

        let qrcode = Path::new(&self.working_dir).join("qrcode.png");

        let mut command = Command::new(&self.vit_kedqr);
        command
            .arg("--pin")
            .arg("1234")
            .arg("--input")
            .arg(private_key_path)
            .arg("--output")
            .arg(qrcode);
        println!("Running vit-kedqr: {:?}", command);
        command.status()?;
        println!("vit-kedqr finished");
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct CardanoKeyTemplate {
    r#type: String,
    description: String,
    #[serde(rename = "cborHex")]
    cbor_hex: String,
}

impl CardanoKeyTemplate {
    pub fn payment_signing_key(cbor_hex: String) -> Self {
        Self {
            r#type: "PaymentSigningKeyShelley_ed25519".to_string(),
            description: "Payment Signing Key".to_string(),
            cbor_hex,
        }
    }

    pub fn payment_verification_key(cbor_hex: String) -> Self {
        Self {
            r#type: "PaymentVerificationKeyShelley_ed25519".to_string(),
            description: "Payment Verification Key".to_string(),
            cbor_hex,
        }
    }

    pub fn stake_signing_key(cbor_hex: String) -> Self {
        Self {
            r#type: "StakeSigningKeyShelley_ed25519".to_string(),
            description: "Stake Signing Key".to_string(),
            cbor_hex,
        }
    }

    pub fn stake_verification_key(cbor_hex: String) -> Self {
        Self {
            r#type: "StakeVerificationKeyShelley_ed25519".to_string(),
            description: "Stake Verification Key".to_string(),
            cbor_hex,
        }
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let content = serde_json::to_string(&self)?;
        write_content(&content, path)
    }
}

fn write_content<P: AsRef<Path>>(content: &str, path: P) -> Result<(), Error> {
    let mut file = std::fs::File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("serialization error")]
    SerializationError(#[from] serde_json::Error),
    #[error("context error")]
    Context(#[from] crate::context::Error),
}
