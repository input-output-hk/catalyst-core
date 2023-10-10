//!
//! Fragment generator
//!

use chain_core::property::FromStr;
use chain_impl_mockchain::{
    certificate::{VoteCast, VotePlanId},
    vote::Payload,
};
use chain_vote::ElectionPublicKey;
use clap::Parser;
use color_eyre::Result;
use lib::fragment::generate_vote_fragment;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

use ed25519_dalek::*;
use std::error::Error;

///
/// Args defines and declares CLI behaviour within the context of clap
///
#[derive(Parser, Debug, Clone)]
#[clap(about, version, author)]
pub struct Args {
    /// Election public key issued by Trent
    #[clap(short, long)]
    pub election_pub_key: String,
    /// Public key of Alice
    #[clap(short, long)]
    public_key: String,
    /// Private key of Alice
    #[clap(short, long)]
    private_key: String,
    /// proposal to vote on
    #[clap(short, long)]
    proposal: u8,
    /// vote plan hash
    #[clap(short, long)]
    vote_plan_id: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    let args = Args::parse();
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

    let pk = hex::decode(args.public_key)?;
    let mut sk = hex::decode(args.private_key)?;
    let election_pk = hex::decode(args.election_pub_key)?;

    // join sk+pk together, api requirement
    sk.extend(pk.clone());
    let keypair: Keypair = Keypair::from_bytes(&sk)?;

    // vote
    let vote = chain_vote::Vote::new(2, 1 as usize).unwrap();
    let crs = chain_vote::Crs::from_hash(args.vote_plan_id.clone().as_bytes());

    // parse ek key
    let ek = ElectionPublicKey::from_bytes(&election_pk)
        .ok_or("unable to parse election pub key".to_string())?;

    // encrypted vote and proof
    let (encrypted_vote, proof) =
        chain_impl_mockchain::vote::encrypt_vote(&mut rng, &crs, &ek, vote);

    // wrap in payload
    let payload = Payload::Private {
        encrypted_vote,
        proof,
    };

    let vote_cast = VoteCast::new(
        VotePlanId::from_str(&args.vote_plan_id.clone())?,
        args.proposal.clone(),
        payload,
    );

    let fragment_bytes = generate_vote_fragment(keypair, vote_cast)?;

    // fragment in hex: output consumed as input to another program
    println!("{:?}", hex::encode(fragment_bytes.clone()));

    Ok(())
}
