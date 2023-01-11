use crate::Result;
use chain_addr::AddressReadable;
use chain_crypto::bech32::Bech32;
use chain_crypto::Ed25519;
use chain_crypto::PublicKey;
use chain_impl_mockchain::vote::CommitteeId;
use jormungandr_lib::interfaces::CommitteeIdDef;
use std::io::stdout;
use std::io::Write;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct CommitteeIdCommandArgs {
    #[clap(short = 'a', long = "address")]
    pub address: Option<String>,

    #[clap(short = 'p', long = "public_key", conflicts_with = "address")]
    pub public_key: Option<String>,

    #[clap(short = 't', long = "testing")]
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
            } else {
                let pkey: PublicKey<Ed25519> =
                    Bech32::try_from_bech32_str(self.public_key.as_ref().unwrap())?;
                CommitteeIdDef::from(CommitteeId::from(pkey))
            }
        };

        writeln!(
            stdout(),
            "{}",
            serde_json::to_string(&committee_id)?
                .as_str()
                .replace("'\"'", "")
        )
        .map_err(Into::into)
    }
}
