//!
//! Community Tally Verification Tool
//!

use chain_vote::tally::batch_decrypt;
use lib::tally::{
    decrypt_tally_with_secret_keys, encode_decrypt_shares, encode_public_keys,
    extract_decrypt_shares, load_decrypt_shares, load_encrypted_tally,
    parse_private_committee_keys, parse_public_committee_keys,
};

use clap::Parser;

use color_eyre::Result;
use tracing::{info, Level};

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

    // Configure a custom event formatter
    let format = tracing_subscriber::fmt::format()
        .with_level(true) // don't include levels in formatted output
        .with_target(true) // don't include targets
        .with_thread_ids(true) // include the thread ID of the current thread
        .with_thread_names(true) // include the name of the current thread
        .compact(); // use the `Compact` formatting style.

    // Create a `fmt` subscriber that uses our custom event format, and set it
    // as the default.
    tracing_subscriber::fmt()
        .event_format(format)
        .with_max_level(Level::INFO /*DEBUG*/)
        .init();

    info!("Audit Tool.");
    info!("Tally Verification Tool.");

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
