use super::Error;
use crate::jcli_lib::utils::{
    vote::{self, SharesError},
    OutputFormat,
};
use chain_vote::tally::{batch_decrypt, EncryptedTally};
use clap::Parser;
use jormungandr_lib::{
    crypto::hash::Hash,
    interfaces::{PrivateTallyState, Tally},
};
use rayon::prelude::*;
use serde::Serialize;
use std::{convert::TryInto, path::PathBuf};

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct TallyVotePlanWithAllShares {
    /// The path to json-encoded vote plan to decrypt. If this parameter is not
    /// specified, the vote plan will be read from the standard
    /// input.
    #[clap(long)]
    vote_plan: Option<PathBuf>,
    /// The id of the vote plan to decrypt.
    /// Can be left unspecified if there is only one vote plan in the input
    #[clap(long)]
    vote_plan_id: Option<Hash>,
    /// The minimum number of shares needed for decryption
    #[clap(long, default_value = "3")]
    threshold: usize,
    /// The path to a JSON file containing decryption shares necessary to decrypt
    /// the vote plan. If this parameter is not specified, the shares will be read
    /// from the standard input.
    #[clap(long)]
    shares: Option<PathBuf>,
    #[clap(flatten)]
    output_format: OutputFormat,
}

#[derive(Serialize)]
struct Output {
    result: Vec<u64>,
}

impl TallyVotePlanWithAllShares {
    pub fn exec(&self) -> Result<(), Error> {
        let mut vote_plan =
            vote::get_vote_plan_by_id(self.vote_plan.as_ref(), self.vote_plan_id.as_ref())?;
        let shares: Vec<Vec<chain_vote::TallyDecryptShare>> =
            vote::read_vote_plan_shares_from_file(
                self.shares.as_ref(),
                vote_plan.proposals.len(),
                Some(self.threshold),
            )?
            .try_into()?;
        let committee_member_keys = vote_plan.committee_member_keys.clone();

        let validated_tallies = (&vote_plan.proposals)
            .into_par_iter()
            .zip(shares.into_par_iter())
            .map(|(proposal, shares)| {
                let encrypted_tally = match &proposal.tally {
                    Tally::Private {
                        state:
                            PrivateTallyState::Encrypted {
                                encrypted_tally, ..
                            },
                    } => encrypted_tally,
                    _ => unreachable!("expected encrypted private tally"),
                };

                let encrypted_tally = EncryptedTally::from_bytes(encrypted_tally.as_ref())
                    .ok_or(Error::EncryptedTallyRead)?;

                encrypted_tally
                    .validate_partial_decryptions(&committee_member_keys, &shares)
                    .map_err(SharesError::ValidationFailed)
                    .map_err(Error::SharesError)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let decrypted_tallies = batch_decrypt(validated_tallies)?;

        for (proposal, decrypted_tally) in vote_plan
            .proposals
            .iter_mut()
            .zip(decrypted_tallies.into_iter())
        {
            proposal.tally = Tally::Private {
                state: PrivateTallyState::Decrypted {
                    result: decrypted_tally.into(),
                },
            }
        }

        let output = self
            .output_format
            .format_json(serde_json::to_value(vote_plan)?)?;
        println!("{}", output);

        Ok(())
    }
}
