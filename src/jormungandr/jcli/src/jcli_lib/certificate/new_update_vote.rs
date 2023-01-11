use crate::jcli_lib::{
    certificate::{write_cert, Error},
    utils::key_parser::parse_pub_key,
};
use chain_crypto::{Ed25519, PublicKey};
use chain_impl_mockchain::certificate::{self, Certificate, UpdateProposalId};
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
pub struct UpdateVote {
    /// the Proposal ID of the proposal.
    #[clap(name = "PROPOSAL_ID")]
    proposal_id: UpdateProposalId,

    /// the voter ID.
    #[clap(name = "VOTER_ID", value_parser = parse_pub_key::<Ed25519>)]
    voter_id: PublicKey<Ed25519>,

    /// print the output signed certificate in the given file, if no file given
    /// the output will be printed in the standard output
    output: Option<PathBuf>,
}

impl UpdateVote {
    pub fn exec(self) -> Result<(), Error> {
        let update_vote = certificate::UpdateVote::new(self.proposal_id, self.voter_id.into());
        let cert = Certificate::UpdateVote(update_vote);
        write_cert(self.output.as_deref(), cert.into())
    }
}
