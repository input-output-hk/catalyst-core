//!
//! Community Tally Verification Tool
//!

use chain_vote::tally::batch_decrypt;

use clap::Parser;

use color_eyre::Result;
use lib::tally::{
    decrypt_tally_with_secret_keys, encode_decrypt_shares, encode_public_keys,
    extract_decrypt_shares, load_decrypt_shares, load_encrypted_tally,
    parse_private_committee_keys, parse_public_committee_keys,
};

use std::error::Error;

///
/// Args defines and declares CLI behaviour within the context of clap
///
#[derive(Parser, Debug, Clone)]
#[clap(about, version, author)]
pub struct Args {
    /// Encrypted tally
    #[clap(short, long)]
    pub encrypted_tally: Option<String>,
    /// Produce decrypt shares: not for public use
    #[clap(
        short,
        long,
        requires = "encrypted_tally",
        value_delimiter = ' ',
        num_args = 1..
    )]
    pub produce_decrypt_shares: Option<Vec<String>>,
    /// Decrypt Tally from shares: public use
    #[clap(
        short,
        long,
        requires = "encrypted_tally",
        requires = "public_keys",
        value_delimiter = ' ',
        num_args = 1..
    )]
    pub decrypt_tally_from_shares: Option<Vec<String>>,
    /// Decrypt Tally from secret keys: internal use
    #[clap(short, long, requires = "encrypted_tally", value_delimiter = ' ', num_args = 1..)]
    pub decrypt_tally_from_keys: Option<Vec<String>>,
    /// List of whitespace seperated Secret keys
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    pub private_keys: Option<Vec<String>>,
    /// List of whitespace seperated Public keys
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    pub public_keys: Option<Vec<String>>,
    /// Show Public keys
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    pub show_public_keys: Option<Vec<String>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    let args = Args::parse();

    //
    // Load decrypt shares for cross referencing Tally result
    // Intended for public use
    //
    if let Some(shares) = args.decrypt_tally_from_shares {
        let shares = load_decrypt_shares(shares)?;
        let pub_keys = args.public_keys.clone().ok_or("handled by clap")?;

        let encrypted_tally =
            load_encrypted_tally(args.encrypted_tally.clone().ok_or("handled by clap")?)?;

        let pks = parse_public_committee_keys(pub_keys)?;

        let validated_tally = encrypted_tally.validate_partial_decryptions(&pks, &shares)?;

        println!(
            "Decryption results {:?}",
            &batch_decrypt([validated_tally])?
        );
    }

    //
    // Produce decrypt shares for publication
    // Internal use only
    //
    if let Some(committee_private_keys) = args.produce_decrypt_shares {
        let (_pub_keys, priv_keys) = parse_private_committee_keys(committee_private_keys)?;

        let encrypted_tally =
            load_encrypted_tally(args.encrypted_tally.clone().ok_or("handled by clap")?)?;

        let shares = extract_decrypt_shares(encrypted_tally, priv_keys);

        println!("Decryption shares: {:?}", encode_decrypt_shares(shares));
    }

    //
    // Decrypt tally from secret keys
    // Intended for internal use
    //
    if let Some(committee_private_keys) = args.decrypt_tally_from_keys {
        let (_pub_keys, priv_keys) = parse_private_committee_keys(committee_private_keys)?;

        let encrypted_tally =
            load_encrypted_tally(args.encrypted_tally.clone().ok_or("handled by clap")?)?;

        println!(
            "tally decryption {:?}",
            decrypt_tally_with_secret_keys(encrypted_tally, priv_keys)
        );
    }

    //
    // Show public keys of private keys
    //
    if let Some(committee_private_keys) = args.show_public_keys {
        let (pub_keys, _priv_keys) = parse_private_committee_keys(committee_private_keys.clone())?;

        let encoded = encode_public_keys(pub_keys)?;
        let it = encoded.iter().zip(committee_private_keys.iter());

        for (i, (x, y)) in it.enumerate() {
            println!("{}: ({}, {})", i, x, y);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{
        encode_decrypt_shares, encode_public_keys, extract_decrypt_shares, load_decrypt_shares,
        load_encrypted_tally, parse_public_committee_keys,
    };
    use rand_chacha::ChaCha20Rng;

    use chain_vote::{tally::batch_decrypt, EncryptedTally};
    use chain_vote::{Ballot, Crs, ElectionPublicKey, MemberCommunicationKey, MemberState, Vote};
    use rand_core::{CryptoRng, RngCore, SeedableRng};

    fn get_encrypted_ballot<R: RngCore + CryptoRng>(
        rng: &mut R,
        pk: &ElectionPublicKey,
        crs: &Crs,
        vote: Vote,
    ) -> Ballot {
        let (enc, proof) = pk.encrypt_and_prove_vote(rng, crs, vote);
        Ballot::try_from_vote_and_proof(enc, &proof, crs, pk).unwrap()
    }

    #[test]
    pub fn test_validation_logic() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let shared_string = b"common reference string (CRS)".to_owned();
        let h = Crs::from_hash(&shared_string);

        let alice = MemberCommunicationKey::new(&mut rng);

        let bob = MemberCommunicationKey::new(&mut rng);

        let charlie = MemberCommunicationKey::new(&mut rng);

        let threshold = 1;

        let alice = MemberState::new(&mut rng, threshold, &h, &[alice.to_public()], 0);
        let bob = MemberState::new(&mut rng, threshold, &h, &[bob.to_public()], 0);
        let charlie = MemberState::new(&mut rng, threshold, &h, &[charlie.to_public()], 0);

        let committee_public_keys =
            vec![alice.public_key(), bob.public_key(), charlie.public_key()];
        let committee_secret_keys = vec![
            alice.member_secret_key(),
            bob.member_secret_key(),
            charlie.member_secret_key(),
        ];

        let ek = ElectionPublicKey::from_participants(&committee_public_keys);

        println!("encrypting vote");

        let vote_options = 2;
        let e1 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 0).unwrap());
        let e2 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 1).unwrap());
        let e3 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 0).unwrap());

        println!("tallying");

        let mut encrypted_tally = EncryptedTally::new(vote_options, ek.clone(), h.clone());
        encrypted_tally.add(&e1, 1);
        encrypted_tally.add(&e2, 3);
        encrypted_tally.add(&e3, 4);

        // Ingredients to publish for community validation (decrypt shares, pub keys, encrypted tally)

        //
        // decrypt shares
        //

        let shares = extract_decrypt_shares(encrypted_tally.clone(), committee_secret_keys);

        let published_shares = encode_decrypt_shares(shares.clone());

        let loaded_shares = load_decrypt_shares(published_shares).unwrap();

        assert_eq!(shares, loaded_shares);

        //
        // encrypted tally
        //

        let published_encrypted_tally = encrypted_tally.to_base64();

        let loaded_encrypted_tally = load_encrypted_tally(published_encrypted_tally).unwrap();

        assert_eq!(encrypted_tally, loaded_encrypted_tally);

        //
        // pub keys
        //

        let pub_keys = encode_public_keys(committee_public_keys.clone()).unwrap();

        let pub_keys = parse_public_committee_keys(pub_keys).unwrap();

        assert_eq!(pub_keys, committee_public_keys);

        //
        // public decryption
        // (decrypt shares, pub keys, encrypted tally)
        //

        let validated_tally = encrypted_tally
            .validate_partial_decryptions(&pub_keys, &shares)
            .unwrap();

        let tally = &batch_decrypt([validated_tally]).unwrap()[0];

        assert_eq!(tally.votes, vec![5, 3]);

        assert!(tally.verify(&encrypted_tally, &committee_public_keys, &shares));

        println!("results from decryption: {:?}", tally);
    }
}
