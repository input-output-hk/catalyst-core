use crate::Result;
use chain_addr::AddressReadable;
use chain_crypto::bech32::Bech32;
use chain_crypto::Ed25519;
use chain_crypto::PublicKey;
use chain_impl_mockchain::vote::CommitteeId;
use jormungandr_lib::interfaces::CommitteeIdDef;
use std::io::stdout;
use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct CommitteeIdCommandArgs {
    /// Careful! directory would be removed before export
    #[structopt(short = "a", long = "address")]
    pub address: Option<String>,

    /// how many addresses to generate
    #[structopt(short = "p", long = "public_key")]
    pub public_key: Option<String>,

    #[structopt(short = "t", long = "testing")]
    pub testing: bool,
}

impl CommitteeIdCommandArgs {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        let committee_id = {
            if let Some(address_str) = self.address {
                let prefix = {
                    if self.testing {
                        "ta"
                    } else {
                        "ca"
                    }
                };
                let address = AddressReadable::from_string(prefix, &address_str)?.to_address();
                CommitteeIdDef::from(CommitteeId::from(address.public_key().unwrap().clone()))
            } else if let Some(public_key) = self.public_key {
                let pkey: PublicKey<Ed25519> = Bech32::try_from_bech32_str(&public_key)?;
                CommitteeIdDef::from(CommitteeId::from(pkey))
            } else {
                println!("no public-key or address provided");
                std::process::exit(-1);
            }
        };

        writeln!(
            stdout(),
            "{}",
            serde_json::to_string(&committee_id)?
                .as_str()
                .replace("\"", "")
        )
        .map_err(Into::into)
    }
}
