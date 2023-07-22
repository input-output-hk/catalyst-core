use chain_addr::{AddressReadable, Discrimination};
use chain_core::{packer::Codec, property::DeserializeFromSlice};

use base64::{engine::general_purpose, Engine};
use chain_impl_mockchain::{
    block::Block, chaintypes::HeaderId, fragment::Fragment, transaction::InputEnum,
};

use chain_vote::TallyDecryptShare;
use color_eyre::Report;
use jormungandr_lib::interfaces::VotePlanStatus;
use jormungandr_lib::interfaces::{AccountIdentifier, Address};

use ::serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use std::collections::HashMap;
use std::error;
use std::{fs::File, path::Path};
use tracing::warn;

use crate::recover::recover_ledger_from_fragments;

const MAIN_TAG: &str = "HEAD";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Storage(#[from] chain_storage::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Csv(#[from] csv::Error),

    #[error("Only accounts inputs are supported not Utxos")]
    UnhandledInput,

    #[error("Unable to extract Tally fragment")]
    CorruptedFragments,
}

#[derive(Serialize, Debug)]
pub struct Vote {
    fragment_id: String,
    caster: Address,
    proposal: u8,
    time: String,
    choice: u8,
    raw_fragment: String,
}

/// Extract fragments from storage
pub fn extract_fragments_from_storage(
    jormungandr_database: &Path,
) -> Result<Vec<Fragment>, Box<dyn error::Error>> {
    let db = chain_storage::BlockStore::file(
        jormungandr_database,
        HeaderId::zero_hash()
            .as_bytes()
            .to_owned()
            .into_boxed_slice(),
    )?;

    let mut fragments = vec![];

    let tip_id = db.get_tag(MAIN_TAG)?.unwrap();
    let distance = db.get_block_info(tip_id.as_ref())?.chain_length();

    let block_iter = db.iter(tip_id.as_ref(), distance)?;

    for iter_res in block_iter {
        let block_bin = iter_res?;
        let mut codec = Codec::new(block_bin.as_ref());
        let block: Block = DeserializeFromSlice::deserialize_from_slice(&mut codec).unwrap();

        for fragment in block.fragments() {
            fragments.push(fragment.to_owned());
        }
    }

    Ok(fragments)
}

/// Replay up until tally, do not include tally fragments
/// State before tally begins i.e encrypted tallies have not been decrypted
/// Tally fragments have been removed
pub fn ledger_before_tally(
    all_fragments: Vec<Fragment>,
    block0: Block,
) -> Result<Vec<VotePlanStatus>, Report> {
    let without_tally_fragments: Vec<Fragment> = all_fragments
        .into_iter()
        .filter(|f| !matches!(f, Fragment::VoteTally(_)))
        .collect();

    let (ledger, failed) =
        recover_ledger_from_fragments(&block0, without_tally_fragments.into_iter())?;
    if !failed.is_empty() {
        warn!("{} fragments couldn't be properly processed", failed.len());
    }

    // recovered ledger is now available for analysis
    let voteplans = ledger.active_vote_plans();
    let offline_voteplans: Vec<VotePlanStatus> =
        voteplans.into_iter().map(VotePlanStatus::from).collect();

    Ok(offline_voteplans)
}

/// Replay all fragments including tally fragments to obtain final decrypted tallies
pub fn ledger_after_tally(
    all_fragments: Vec<Fragment>,
    block0: Block,
) -> Result<Vec<VotePlanStatus>, Report> {
    let (ledger, failed) = recover_ledger_from_fragments(&block0, all_fragments.into_iter())?;
    if !failed.is_empty() {
        warn!("{} fragments couldn't be properly processed", failed.len());
    }

    // recovered ledger is now available for analysis
    let voteplans = ledger.active_vote_plans();
    let offline_voteplans: Vec<VotePlanStatus> =
        voteplans.into_iter().map(VotePlanStatus::from).collect();

    Ok(offline_voteplans)
}

/// Extract decryption shares and results from tally fragments
pub fn extract_decryption_shares_and_results(
    all_fragments: Vec<Fragment>,
) -> HashMap<String, Vec<String>> {
    let mut shares_and_results: HashMap<String, Vec<String>> = HashMap::new();
    let tally_fragments: Vec<Fragment> = all_fragments
        .into_iter()
        .filter(|f| matches!(f, Fragment::VoteTally(_)))
        .collect();

    for fragment in tally_fragments {
        if let Fragment::VoteTally(tx) = fragment {
            let certificate = tx.as_slice().payload().into_payload();

            if let Some(dt) = certificate.tally_decrypted() {
                for tally in dt.iter() {
                    let decrypt_shares =
                        decrypt_shares_to_b64(tally.clone().decrypt_shares.into_vec());
                    let results = tally.clone().tally_result.into_vec();
                    shares_and_results.insert(format!("{:?}", results), decrypt_shares);
                }
            }
        }
    }
    shares_and_results
}

// decrypt_shares_to_b64 converts decrypt shares to base64
fn decrypt_shares_to_b64(decrypt_shares: Vec<TallyDecryptShare>) -> Vec<String> {
    let mut shares = vec![];
    for share in decrypt_shares {
        shares.push(general_purpose::STANDARD.encode(share.to_bytes()));
    }

    shares
}

/// Did I vote?
/// Iterate through all vote cast fragments and match the given voters pub key to confirm vote "went through".
///
pub fn find_vote(jormungandr_database: &Path, caster_address: String) -> Result<Vec<Vote>, Error> {
    let db = chain_storage::BlockStore::file(
        jormungandr_database,
        HeaderId::zero_hash()
            .as_bytes()
            .to_owned()
            .into_boxed_slice(),
    )?;

    let caster_address = AddressReadable::from_string(&"ca".to_string(), &caster_address)
        .unwrap()
        .to_address();

    // Tag should be present
    let tip_id = db.get_tag(MAIN_TAG)?.unwrap();
    let distance = db.get_block_info(tip_id.as_ref())?.chain_length();

    let mut votes = vec![];

    let block_iter = db.iter(tip_id.as_ref(), distance)?;

    for iter_res in block_iter {
        let block_bin = iter_res?;
        let mut codec = Codec::new(block_bin.as_ref());
        let block: Block = DeserializeFromSlice::deserialize_from_slice(&mut codec).unwrap();

        for fragment in block.fragments() {
            if let Fragment::VoteCast(tx) = fragment {
                let fragment_id = fragment.hash();

                let input = tx.as_slice().inputs().iter().next().unwrap().to_enum();
                let caster = if let InputEnum::AccountInput(account_id, _value) = input {
                    AccountIdentifier::from(account_id)
                        .into_address(Discrimination::Production, "ca")
                } else {
                    return Err(Error::UnhandledInput);
                };
                let certificate = tx.as_slice().payload().into_payload();

                let choice = match certificate.payload() {
                    chain_impl_mockchain::vote::Payload::Public { choice } => choice.as_byte(),
                    chain_impl_mockchain::vote::Payload::Private { .. } => {
                        // zeroing data to enable private voting support
                        // (at least everying exception choice, since it is disabled by design in private vote)
                        0u8
                    }
                };

                let v = Vote {
                    fragment_id: fragment_id.to_string(),
                    caster: caster.clone(),
                    proposal: certificate.proposal_index(),
                    time: block.header().block_date().to_string(),
                    raw_fragment: hex::encode(tx.as_ref()),
                    choice,
                };

                if caster.clone() == caster_address.clone().into() {
                    votes.push(v);
                }
            }
        }
    }
    Ok(votes)
}

pub fn json_from_file<T: for<'a> Deserialize<'a>>(path: impl AsRef<Path>) -> color_eyre::Result<T> {
    Ok(serde_json::from_reader(File::open(path)?)?)
}

pub fn deserialize_truthy_falsy<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let truthy_value: String = String::deserialize(deserializer)?;
    Ok(matches!(
        truthy_value.to_lowercase().as_ref(),
        "x" | "1" | "true"
    ))
}

#[cfg(test)]
mod tests {
    use crate::offline::extract_fragments_from_storage;

    use std::path::PathBuf;

    use super::find_vote;

    #[test]
    #[ignore]
    fn test_fragments_extraction() {
        // Everytime the test is run, the storage folder get overwritten.
        // If you are getting weird errors, tar -xvf to new folder and point path accordingly to reset state.
        let path = PathBuf::from("/tmp/fund9-leader-1/persist/leader-1");

        let _fragments = extract_fragments_from_storage(&path).unwrap();
    }

    #[test]
    #[ignore]
    fn test_find_vote() {
        let path = PathBuf::from("/tmp/fund9-leader-1/persist/leader-1");

        // ed25519 public key in bech32 format
        let pub_key = "ca1qkgkj2twpl77c44nv06zkueuptwn93u5zmcx7dl37vnk5cehyj44jy3nush".to_string();

        let votes = find_vote(&path, pub_key).unwrap();

        println!("votes for voter{:?}", votes);
    }
}
