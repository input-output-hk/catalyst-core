//!
//! Fragment generator
//!

use bech32::Error as Bech32Error;
use bech32::FromBase32;
use chain_vote::ElectionPublicKey;
use clap::Parser;
use color_eyre::Result;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

use ed25519_dalek::*;
use std::error::Error;

use crate::fragment::{compose_encrypted_vote_part, generate_vote_fragment};

mod fragment;

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
    /// Epoch
    #[clap(short, long)]
    epoch: u32,
    /// Slot
    #[clap(short, long)]
    slot: u32,
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

    // Election pub key published as a Bech32_encoded address
    // which consists of 3 parts: A Human-Readable Part (HRP) + Separator + Data:
    let (_hrp, data, _variant) =
        bech32::decode(&args.election_pub_key).map_err(Bech32Error::from)?;

    let election_pk = Vec::<u8>::from_base32(&data).map_err(Bech32Error::from)?;

    // join sk+pk together, api requirement
    sk.extend(pk.clone());
    let keypair: Keypair = Keypair::from_bytes(&sk)?;

    // vote
    let vote = chain_vote::Vote::new(2, 1_usize)?;
    let crs = chain_vote::Crs::from_hash(args.vote_plan_id.clone().as_bytes());

    // parse ek key
    let ek = ElectionPublicKey::from_bytes(&election_pk)
        .ok_or("unable to parse election pub key".to_string())?;

    let (ciphertexts, proof) = ek.encrypt_and_prove_vote(&mut rng, &crs, vote);
    let (proof, encrypted_vote) = compose_encrypted_vote_part(ciphertexts.clone(), proof)?;

    let fragment_bytes = generate_vote_fragment(
        keypair,
        encrypted_vote,
        proof,
        args.proposal,
        &hex::decode(args.vote_plan_id)?,
        args.epoch,
        args.slot,
    )?;

    // fragment in hex: output consumed as input to another program
    println!("{:?}", hex::encode(fragment_bytes.clone()));

    Ok(())
}
