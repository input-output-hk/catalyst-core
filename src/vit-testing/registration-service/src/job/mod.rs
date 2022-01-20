mod info;

use crate::config::NetworkType;
use crate::request::Request;
use crate::utils::CommandExt as _;
pub use info::JobOutputInfo;
use jormungandr_automation::jcli::JCli;
use jortestkit::prelude::read_file;
use jortestkit::prelude::ProcessOutput;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitStatus;
use std::str::FromStr;
use thiserror::Error;

const PIN: &str = "1234";

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

impl Default for VoteRegistrationJobBuilder {
    fn default() -> Self {
        Self::new()
    }
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
        stake_verification_key: P,
        output: Q,
    ) -> Result<ExitStatus, Error> {
        let mut command = Command::new(&self.cardano_cli);
        command
            .arg("address")
            .arg("build")
            .arg("--verification-key-file")
            .arg(verification_key.as_ref())
            .arg("--stake-verification-key-file")
            .arg(stake_verification_key.as_ref())
            .arg("--out-file")
            .arg(output.as_ref())
            .arg_network(self.network);
        println!("generate addres: {:?}", command);
        command.status().map_err(Into::into)
    }

    pub fn generate_stake_address<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        verification_key: P,
        output: Q,
    ) -> Result<ExitStatus, Error> {
        let mut command = Command::new(&self.cardano_cli);
        command
            .arg("stake-address")
            .arg("build")
            .arg("--stake-verification-key-file")
            .arg(verification_key.as_ref())
            .arg("--out-file")
            .arg(output.as_ref())
            .arg_network(self.network);
        println!("generate stake-address: {:?}", command);
        command.status().map_err(Into::into)
    }

    pub fn start(&self, request: Request) -> Result<JobOutputInfo, Error> {
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
        stake_skey.write_to_file(&stake_skey_path)?;
        println!("stake.skey saved");

        println!("saving stake.vkey...");
        let stake_vkey = CardanoKeyTemplate::stake_verification_key(request.stake_vkey);
        let stake_vkey_path = Path::new(&self.working_dir).join("stake.vkey");
        stake_vkey.write_to_file(&stake_vkey_path)?;
        println!("stake.vkey saved");

        let jcli = JCli::new(self.jcli.clone());
        let private_key = if let Some(key) = request.vote_skey {
            key
        } else {
            jcli.key().generate_default()
        };

        println!("saving catalyst-vote.skey...");
        let private_key_path = Path::new(&self.working_dir).join("catalyst-vote.skey");
        write_content(&private_key, &private_key_path)?;
        println!("catalyst-vote.skey saved");

        println!("saving catalyst-vote.pkey...");
        let public_key = jcli.key().convert_to_public_string(&private_key);
        let public_key_path = Path::new(&self.working_dir).join("catalyst-vote.pkey");
        write_content(&public_key, &public_key_path)?;
        println!("catalyst-vote.pkey saved");

        println!("saving rewards.addr...");
        let rewards_address_path = Path::new(&self.working_dir).join("rewards.addr");
        self.generate_stake_address(&stake_vkey_path, &rewards_address_path)?;
        println!("rewards.addr saved");

        let rewards_address = read_file(&rewards_address_path);
        println!("rewards.addr: {}", rewards_address);

        println!("saving payment.addr...");
        let payment_address_path = Path::new(&self.working_dir).join("payment.addr");
        self.generate_payment_address(&payment_vkey_path, &stake_vkey_path, &payment_address_path)?;
        println!("payment.addr saved");

        let payment_address = read_file(&payment_address_path);
        println!("payment.addr: {}", payment_address);

        let mut command = Command::new(&self.cardano_cli);
        command
            .arg("query")
            .arg("utxo")
            .arg_network(self.network)
            .arg("--address")
            .arg(&payment_address)
            .arg("--out-file")
            .arg("/dev/stdout");

        println!("Running cardano_cli: {:?}", command);
        let funds = get_funds(command.output()?.as_lossy_string())?;
        println!("cardano_cli finished");

        let vote_registration_path = Path::new(&self.working_dir).join("vote-registration.tx");

        let mut command = Command::new(&self.voter_registration);
        command
            .arg("--payment-signing-key")
            .arg(&payment_skey_path)
            .arg("--payment-address")
            .arg(&payment_address)
            .arg("--rewards-address")
            .arg(&rewards_address)
            .arg("--stake-signing-key")
            .arg(&stake_skey_path)
            .arg("--vote-public-key")
            .arg(&public_key_path)
            .arg_network(self.network)
            .arg("--mary-era")
            .arg("--cardano-mode")
            .arg("--sign")
            .arg("--out-file")
            .arg(&vote_registration_path);

        println!("Running voter-registration: {:?}", command);

        let output = command.output()?;

        println!("status: {}", output.status);
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();

        let slot_no = get_slot_no(output.as_multi_line())?;
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

        let qrcode = Path::new(&self.working_dir).join(format!("qrcode_pin_{}.png", PIN));

        let mut command = Command::new(&self.vit_kedqr);
        command
            .arg("--pin")
            .arg(PIN)
            .arg("--input")
            .arg(private_key_path)
            .arg("--output")
            .arg(qrcode);
        println!("Running vit-kedqr: {:?}", command);
        command.status()?;
        println!("vit-kedqr finished");

        Ok(JobOutputInfo { slot_no, funds })
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
    #[error("cannot parse voter-registration output: {0:?}")]
    CannotParseVoterRegistrationOutput(Vec<String>),
    #[error("cannot parse cardano cli output: {0:?}")]
    CannotParseCardanoCliOutput(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct FundsResponse {
    #[serde(flatten)]
    content: HashMap<String, FundsEntry>,
}

impl FundsResponse {
    pub fn get_first(&self) -> Result<FundsEntry, Error> {
        Ok(self
            .content
            .iter()
            .next()
            .ok_or_else(|| Error::CannotParseCardanoCliOutput("empty response".to_string()))?
            .1
            .clone())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct FundsEntry {
    address: String,
    value: FundsValue,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct FundsValue {
    lovelace: u64,
}

/// Supported output
/// {
///     "bf810bff3f979e28441775077524d41741542d1f237b0b7d6a698164dcded29b#0": {
///         "address": "addr_test1vqtsh379yx9hmn8ypedzx270q9ueea24suf3zu85p2mv2sgra46ef",
///         "value": {
///             "lovelace": 9643918
///         }
///     }
/// }
pub fn get_funds(output: String) -> Result<u64, Error> {
    println!("get_funds: {}", output);
    let response: FundsResponse =
        serde_json::from_str(&output).map_err(|_| Error::CannotParseCardanoCliOutput(output))?;
    Ok(response.get_first()?.value.lovelace)
}

/// Supported output:
/// Vote public key used        (hex): c6b6d184ea26781f00b9034ec0ba974f2f833788ce2e24cc37e9e8f41131e1fa
/// Stake public key used       (hex): e542b6a0ced80e1ab5bda70311bf643b9011ee04411737f3e0136825ef47f2d8
/// Rewards address used        (hex): 60170bc7c5218b7dcce40e5a232bcf01799cf55587131170f40ab6c541
/// Slot registered:                   25398498
/// Vote registration signature (hex): e5cc2e1a9344794cbad76bb65d485776aa560baca6133cdfe77827b15dd0e4c883c32e7177dc15d55e34f79df7ffaebca4d271271c6615b0dacc90e36fb22f03
pub fn get_slot_no(output: Vec<String>) -> Result<u64, Error> {
    output
        .iter()
        .find(|x| x.contains("Slot registered"))
        .ok_or_else(|| Error::CannotParseVoterRegistrationOutput(output.clone()))?
        .split_whitespace()
        .nth(2)
        .ok_or_else(|| Error::CannotParseVoterRegistrationOutput(output.clone()))?
        .parse()
        .map_err(|_| Error::CannotParseVoterRegistrationOutput(output.clone()))
}

#[cfg(test)]
mod tests {

    use super::{get_funds, get_slot_no};

    #[test]
    pub fn test_funds_extraction() {
        let content = vec![
        "{", 
        "    \"bf810bff3f979e28441775077524d41741542d1f237b0b7d6a698164dcded29b#0\": {",
        "        \"address\": \"addr_test1vqtsh379yx9hmn8ypedzx270q9ueea24suf3zu85p2mv2sgra46ef\",",
        "        \"value\": {", 
        "            \"lovelace\": 9643918", 
        "        }", 
        "    }", 
        "}"];
        assert_eq!(get_funds(content.join(" ")).unwrap(), 9643918);
    }

    #[test]
    pub fn test_slot_no_extraction() {
        let content = vec![
            "Vote public key used        (hex): c6b6d184ea26781f00b9034ec0ba974f2f833788ce2e24cc37e9e8f41131e1fa".to_string(),
            "Stake public key used       (hex): e542b6a0ced80e1ab5bda70311bf643b9011ee04411737f3e0136825ef47f2d8".to_string(),
            "Rewards address used        (hex): 60170bc7c5218b7dcce40e5a232bcf01799cf55587131170f40ab6c541".to_string(),
            "Slot registered:                   25398498".to_string(),
            "Vote registration signature (hex): e5cc2e1a9344794cbad76bb65d485776aa560baca6133cdfe77827b15dd0e4c883c32e7177dc15d55e34f79df7ffaebca4d271271c6615b0dacc90e36fb22f03".to_string()
        ];

        assert_eq!(get_slot_no(content).unwrap(), 25398498);
    }
}
