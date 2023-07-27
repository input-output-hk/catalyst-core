use chain_core::{packer::Codec, property::DeserializeFromSlice};

use base64::{engine::general_purpose, Engine};
use chain_impl_mockchain::{block::Block, chaintypes::HeaderId, fragment::Fragment};

use chain_vote::TallyDecryptShare;
use color_eyre::Report;
use jormungandr_lib::interfaces::Address;
use jormungandr_lib::interfaces::VotePlanStatus;

use ::serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use std::collections::HashMap;
use std::error;
use std::{fs::File, path::Path};
use tracing::{error, warn};

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
    pub fragment_id: String,
    pub caster: Address,
    pub proposal: u8,
    pub time: String,
    pub choice: u8,
    pub raw_fragment: String,
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
    let without_tally_fragments = all_fragments
        .into_iter()
        .filter(|f| !matches!(f, Fragment::VoteTally(_)));

    let (ledger, failed) = recover_ledger_from_fragments(&block0, without_tally_fragments)?;
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

    #[test]
    #[ignore]
    fn test_fragments_extraction() {
        // Test over writes storage folder.
        // If you are getting weird errors, tar -xvf to new folder and point path accordingly to reset state.
        let path = PathBuf::from("/tmp/fund9-leader-1/persist/leader-1");

        let _fragments = extract_fragments_from_storage(&path).unwrap();
    }
}
