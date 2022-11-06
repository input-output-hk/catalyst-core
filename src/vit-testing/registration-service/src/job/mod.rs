mod builder;
mod info;

use crate::cardano::CardanoCli;
use crate::cardano::CardanoKeyTemplate;
use crate::catalyst_toolbox::CatalystToolboxCli;
use crate::config::NetworkType;
use crate::request::Request;
use crate::utils::write_content;
use crate::Error;
use crate::VoterRegistrationCli;
pub use builder::VoteRegistrationJobBuilder;
pub use info::JobOutputInfo;
use jormungandr_automation::jcli::JCli;
use jortestkit::prelude::read_file;
use scheduler_service_lib::JobRunner;
use std::path::{Path, PathBuf};

pub struct VoteRegistrationJob {
    pub(super) jcli: PathBuf,
    pub(super) cardano_cli: CardanoCli,
    pub(super) voter_registration: VoterRegistrationCli,
    pub(super) catalyst_toolbox: CatalystToolboxCli,
    pub(super) network: NetworkType,
}

impl JobRunner<Request, JobOutputInfo, Error> for VoteRegistrationJob {
    fn start(
        &self,
        request: Request,
        working_dir: PathBuf,
    ) -> Result<Option<JobOutputInfo>, Error> {
        println!("saving payment.skey...");
        let payment_skey = CardanoKeyTemplate::payment_signing_key(request.payment_skey.clone());
        let payment_skey_path = Path::new(&working_dir).join("payment.skey");
        payment_skey.write_to_file(&payment_skey_path)?;
        println!("payment.skey saved");

        println!("saving payment.vkey...");
        let payment_vkey =
            CardanoKeyTemplate::payment_verification_key(request.payment_vkey.clone());
        let payment_vkey_path = Path::new(&working_dir).join("payment.vkey");
        payment_vkey.write_to_file(&payment_vkey_path)?;
        println!("payment.vkey saved");

        println!("saving stake.skey...");
        let stake_skey = CardanoKeyTemplate::stake_signing_key(request.stake_skey.clone());
        let stake_skey_path = Path::new(&working_dir).join("stake.skey");
        stake_skey.write_to_file(&stake_skey_path)?;
        println!("stake.skey saved");

        println!("saving stake.vkey...");
        let stake_vkey = CardanoKeyTemplate::stake_verification_key(request.stake_vkey.clone());
        let stake_vkey_path = Path::new(&working_dir).join("stake.vkey");
        stake_vkey.write_to_file(&stake_vkey_path)?;
        println!("stake.vkey saved");

        println!("saving rewards.addr...");
        let rewards_address_path = Path::new(&working_dir).join("rewards.addr");
        self.cardano_cli.stake_address().build(
            &stake_vkey_path,
            &rewards_address_path,
            self.network,
        )?;
        println!("rewards.addr saved");

        let rewards_address = read_file(&rewards_address_path)?;
        println!("rewards.addr: {}", rewards_address);

        println!("saving payment.addr...");
        let payment_address_path = Path::new(&working_dir).join("payment.addr");
        self.cardano_cli.address().build(
            &payment_vkey_path,
            &stake_vkey_path,
            &payment_address_path,
            self.network,
        )?;
        println!("payment.addr saved");

        let payment_address = read_file(&payment_address_path)?;
        println!("payment.addr: {}", payment_address);

        let utxo = self
            .cardano_cli
            .query()
            .utxo(&payment_address, self.network)?;

        let slot: u64 = self.cardano_cli.query().tip(self.network)?.slot;

        let metadata_path = Path::new(&working_dir).join("metadata.json");

        let voter_registration = self.voter_registration.clone();

        if request.is_legacy() {
            let jcli = JCli::new(self.jcli.clone());

            let private_key = request
                .legacy_skey
                .unwrap_or_else(|| (jcli.key().generate_default()));

            println!("saving catalyst-vote.skey...");
            let private_key_path = Path::new(&working_dir).join("catalyst-vote.skey");
            write_content(&private_key, &private_key_path)?;
            println!("catalyst-vote.skey saved");

            println!("saving catalyst-vote.pkey...");
            let public_key = jcli.key().convert_to_public_string(&private_key);
            let public_key_path = Path::new(&working_dir).join("catalyst-vote.pkey");
            write_content(&public_key, &public_key_path)?;
            println!("catalyst-vote.pkey saved");

            println!("generating metadata");
            voter_registration.generate_legacy_metadata(
                rewards_address,
                public_key_path,
                stake_skey_path,
                slot,
                metadata_path,
            )?;

            let pin = "1234".to_string();
            let qr_code = Path::new(&working_dir).join(format!("qr_code_{}.png", pin));

            self.catalyst_toolbox
                .generate_qr(private_key_path, pin, qr_code)?;
        } else {
            voter_registration.generate_delegation_metadata(
                rewards_address,
                request.delegations(),
                stake_skey_path,
                slot,
                metadata_path,
            )?;
        }

        println!("retrieving protocol parameters");
        let protocol_parameters = Path::new(&working_dir).join("protocol_parameters.json");
        self.cardano_cli
            .query()
            .protocol_parameters(self.network, &protocol_parameters)?;

        println!("registering certificate");
        let stake_cert = Path::new(&working_dir).join("stake.cert");
        self.cardano_cli
            .stake_address()
            .register_certificate(&stake_vkey_path, &stake_cert)?;

        println!("building transaction..");
        let tx_raw = Path::new(&working_dir).join("tx.raw");
        self.cardano_cli.transaction().build(
            self.network,
            utxo.entries.first().unwrap().hash.clone(),
            payment_address,
            stake_cert,
            protocol_parameters,
            tx_raw.clone(),
            2,
        )?;

        let tx_signed = Path::new(&working_dir).join("tx.signed");
        println!("signing transaction..");
        self.cardano_cli
            .transaction()
            .sign(&tx_raw, &payment_skey_path, &tx_signed)?;
        println!("submitting transaction..");
        self.cardano_cli.transaction().submit(&tx_signed)?;

        Ok(Some(JobOutputInfo {
            slot_no: slot,
            funds: utxo.get_total_funds(),
        }))
    }
}
