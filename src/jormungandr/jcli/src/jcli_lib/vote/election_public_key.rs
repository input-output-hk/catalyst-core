use crate::jcli_lib::vote::{Error, OutputFile};
use chain_crypto::bech32::Bech32;
use std::io::Write as _;
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct ElectionPublicKey {
    /// Keys of all committee members
    #[clap(
        value_parser = chain_vote::committee::MemberPublicKey::try_from_bech32_str,
        required = true,
        short = 'k',
        long = "keys"
    )]
    member_keys: Vec<chain_vote::committee::MemberPublicKey>,

    #[clap(flatten)]
    output_file: OutputFile,
}

impl ElectionPublicKey {
    pub fn exec(&self) -> Result<(), Error> {
        let election_public_key =
            chain_vote::ElectionPublicKey::from_participants(&self.member_keys);

        let mut output = self.output_file.open()?;
        writeln!(output, "{}", election_public_key.to_bech32_str()).map_err(Error::from)
    }
}
