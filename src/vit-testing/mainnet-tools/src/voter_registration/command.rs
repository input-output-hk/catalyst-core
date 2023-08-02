use super::fake;
use clap::Parser;
use std::path::PathBuf;
use thiserror::Error;

pub const PATH_TO_DYNAMIC_CONTENT: &str = "VOTER_REGISTRATION_DYNAMIC_CONTENT";

/// Voter registration mock. It can return correctly formatted but faked response similar to
/// Haskel voter registration CLI: <https://github.com/input-output-hk/voting-tools/tree/master/registration>
#[derive(Parser, Debug)]
pub struct Command {
    /// Rewards address in bech32
    #[clap(long = "rewards-address")]
    pub rewards_address: String,
    /// Path to catalyst voting key file in bech32
    #[clap(long = "vote-public-key-file")]
    pub vote_public_key_file: PathBuf,
    /// Path to cardano stake signing key file in bech32
    #[clap(long = "stake-signing-key-file")]
    pub stake_signing_key_file: PathBuf,
    /// Slot number which will be used as a nonce
    #[clap(long = "slot-no")]
    pub slot_no: u32,
}

impl Command {
    /// Executes command
    ///
    /// # Errors
    ///
    /// On missing parameters or IO related errors
    pub fn exec(self) -> Result<(), Error> {
        if !self.stake_signing_key_file.exists() {
            return Err(Error::StakeSigningKey);
        }
        if !self.vote_public_key_file.exists() {
            return Err(Error::VotePublicKey);
        }

        let output = match std::env::var(PATH_TO_DYNAMIC_CONTENT) {
            Ok(value) => std::fs::read_to_string(value)?,
            Err(_) => fake::metadata(),
        };
        println!("{output}");
        Ok(())
    }
}

/// Errors for Voter Registration Mock
#[derive(Error, Debug)]
pub enum Error {
    /// Payment key IO related error
    #[error("payment-signing-key: file does not exists")]
    PaymentSigningKey,
    /// Stake signing key IO related error
    #[error("stake-signing-key: file does not exists")]
    StakeSigningKey,
    /// Catalyst key IO related error
    #[error("vote-public-key: file does not exists")]
    VotePublicKey,
    /// General IO related error
    #[error("cannot create output file")]
    IoError(#[from] std::io::Error),
}
