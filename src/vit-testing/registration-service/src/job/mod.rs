use std::io::Write;
       

pub struct VoteRegistrationJobBuilder{
    job: VoteRegistrationJob
}

impl VoteRegistrationJobBuilder {
    pub fn new() -> Self {
        Self {
            job: Default::default()
        }
    }

    pub fn with_jcli<P: AsRef<Path>>(self,jcli: P) -> Self {
        self.job.jcli = jcli.as_ref().to_path_buf();
        self
    }

    pub fn with_cardano_cli<P: AsRef<Path>>(self,cardano_cli: P) -> Self {
        self.job..cardano_cli = cardano_cli.as_ref().to_path_buf();
        self
    }

    pub fn with_voter_registration<P: AsRef<Path>>(self,voter_registration: P) -> Self {
        self.job..voter_registration = voter_registration.as_ref().to_path_buf();
        self
    }

    pub fn with_network(self,network: NetworkType) -> Self {
        self.job..network = network;
        self
    }

    pub fn with_working_dir<P: AsRef<Path>>(self,working_dir: P) -> Self {
        self.job..working_dir = working_dir.as_ref().to_path_buf();
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
    Network: NetworkType,
    working_dir: PathBuf
}

impl Default for VoteRegistrationJob {
    fn default() -> Self {
        Self {
            jcli: PathBuf::from_str("jcli"),
            cardano_cli:  PathBuf::from_str("cardano-cli"),
            voter_registration:  PathBuf::from_str("voter-registration"),
            payment_signing_key:  PathBuf::from_str("./payment.skey"),
            stake_signing_key:  PathBuf::from_str("./staking.skey"),
            Network: NetworkType::Mainnet,
            temp_dir:  PathBuf::from_str(".")
        }
    }
}

impl VoteRegistrationJob {

    pub fn generate_payment_address<P: AsRef<Path>, Q: AsRef<Path>>(&self, verification_key: P, output: Q) -> Result<Status,CardanoCliError>{
        Command::new(self.cardano_cli).arg("address").arg("build").arg("--verification-key-file")
            .arg(verification_key.as_ref()).arg("--out-file").arg(output.as_ref())
            .arg("--mainnet")
            .status()
    }

    pub fn start(&self, request: Request) {
        let payment_skey = CardanoKeyTemplate::payment_signing_key(request.payment_skey);
        let payment_skey_path = Path::new(&self.working_dir).join("payment.skey")
        payment_skey.write_to_file(payment_skey_path);
        
        let payment_vkey = CardanoKeyTemplate::payment_signing_key(request.payment_vkey);
        let payment_vkey_path = Path::new(&self.working_dir).join("payment.vkey");
        payment_skey.write_to_file(payment_vkey_path);
        

        let stake_skey = CardanoKeyTemplate::payment_signing_key(request.stake_skey);
        let stake_skey_path = Path::new(&self.working_dir).join("stake.skey");
        stake_skey.write_to_file(stake_skey_path);
        
        let stake_vkey = CardanoKeyTemplate::payment_signing_key(request.stake_vkey);
        let stake_vkey_path = Path::new(&self.working_dir).join("stake.vkey"));
        stake_vkey.write_to_file(stake_vkey_path);

        let jcli = JCli::new(self.jcli);
        let private_key = jcli.key().generate_default();
        let private_key_path = Path::new(&self.working_dir).join("catalyst-vote.skey");
        write_to_file(private_key,private_key_path);

        let public_key = jcli.key().convert_to_public_string(&private_key);
        let public_key_path = Path::new(&self.working_dir).join("catalyst-vote.pkey");
        write_to_file(public_key,public_key_path);
        
        let payment_address_path = Path::new(&self.working_dir).join("payment.addr");
        generate_payment_address(payment_skey_path,payment_address_path)?;

        let vote_registration_path = Path::new(&self.working_dir).join("vote-registration.tx"); 

        Command::new(self.voter_registration)
            .arg("--payment-signing-key").arg(payment_skey_path)
            .arg("--payment-address").arg(payment_skey_path)
            .arg("--stake-signing-key").arg(payment_skey_path)
            .arg("--vote-public-key").arg(public_key_path)
            .arg("--mainnet")
            .arg("--mary-era")
            .arg("--cardano-mode")
            .arg("--sign")
            .arg("--out-file")
            .arg(vote_registration_path)
            .status()?;

        Command::new(self.cardano_cli).arg("transaction").arg("submit")
            .arg("--cardano-mode").arg("--mainnet").arg("--tx-file")
            .arg(vote_registration_path)
            .status()?;

        let qrcode = Path::new(&self.working_dir).join("qrcode.png");

        Command::new(self.vit_kedqr).arg("-pin").arg("1234").arg("-input")
            .arg(private_key_path)
            .arg("-output")
            .arg(qrcode);
            /*
            /voter-registration  --payment-signing-key somepayment.skey \
                      --payment-address $(cat somepayment.addr) \
                      --vote-public-key catalyst-vote.pkey \
                      --stake-signing-key pledge.staking.skey \
                      --mainnet \
                      --mary-era \
                      --cardano-mode \
                      --sign \
                      --out-file vote-registration.tx
                      */
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct CardanoKeyTemplate{
    type: String,
    description: String,
    cborHex: String
}

impl CardanoKeyTemplate {
    pub fn payment_signing_key(cbor_hex: String) -> Self {
        Self {
            type: "PaymentSigningKeyShelley_ed25519".to_string(),
            description: "Payment Signing Key",
            cbor_hex
        }
    } 

    pub fn payment_verification_key(cbor_hex: String) -> Self {
        Self {
            type: "PaymentVerificationKeyShelley_ed25519".to_string(),
            description: "Payment Verification Key".to_string(),
            cbor_hex
        }        
    } 

    pub fn stake_signing_key(cbor_hex: String) -> Self {
        Self {
            type: "StakeSigningKeyShelley_ed25519".to_string(),
            description: "Stake Signing Key".to_string(),
            cbor_hex
        }        
    } 

    pub fn stake_verification_key(cbor_hex: String) -> Self {
        Self {
            type: "StakeVerificationKeyShelley_ed25519".to_string(),
            description: "Stake Verification Key".to_string(),
            cbor_hex
        }        
    } 

    pub fn write_to_file<P: AsRef<Path>>(path: P) -> Result<(),std::io::Error>{
        let content = serde_yaml::to_string(&self)?;
        write_content(&content)
    }
}

fn write_content<P: AsRef<Path>>(content: &str, path:P) {
    let mut file = std::fs::File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

