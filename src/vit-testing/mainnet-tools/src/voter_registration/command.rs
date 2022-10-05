use super::fake;
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

pub const PATH_TO_DYNAMIC_CONTENT: &str = "VOTER_REGISTRATION_DYNAMIC_CONTENT";

#[derive(StructOpt, Debug)]
pub struct VoterRegistrationCommand {
    #[structopt(long = "rewards-address")]
    pub rewards_address: String,

    #[structopt(long = "vote-public-key-file")]
    pub vote_public_key_file: PathBuf,

    #[structopt(long = "stake-signing-key-file")]
    pub stake_signing_key_file: PathBuf,

    #[structopt(long = "slot-no")]
    pub slot_no: u32,

    #[structopt(long = "json")]
    pub json: bool,
}

impl VoterRegistrationCommand {
    pub fn exec(self) -> Result<(), Error> {
        if !self.stake_signing_key_file.exists() {
            return Err(Error::StakeSigningKey);
        }
        if !self.vote_public_key_file.exists() {
            return Err(Error::VotePublicKey);
        }

        let output = match std::env::var(PATH_TO_DYNAMIC_CONTENT) {
            Ok(value) => std::fs::read_to_string(value).unwrap(),
            Err(_) => fake::metadata(),
        };
        println!("{}", output);
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("payment-signing-key: file does not exists")]
    PaymentSigningKey,
    #[error("stake-signing-key: file does not exists")]
    StakeSigningKey,
    #[error("vote-public-key: file does not exists")]
    VotePublicKey,
    #[error("cannot create output file")]
    IoError(#[from] std::io::Error),
}
