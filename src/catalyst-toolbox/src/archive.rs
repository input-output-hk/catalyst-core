use chain_addr::Discrimination;
use chain_core::{packer::Codec, property::Deserialize};
use chain_impl_mockchain::{
    block::Block, chaintypes::HeaderId, fragment::Fragment, transaction::InputEnum,
};
use jormungandr_lib::interfaces::{AccountIdentifier, Address};

use serde::Serialize;

use std::{collections::HashMap, path::Path};

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
}

#[derive(Serialize)]
struct Vote {
    fragment_id: String,
    caster: Address,
    proposal: u8,
    time: String,
    choice: u8,
    raw_fragment: String,
}

pub fn generate_archive_files(jormungandr_database: &Path, output_dir: &Path) -> Result<(), Error> {
    let db = chain_storage::BlockStore::file(
        jormungandr_database,
        HeaderId::zero_hash()
            .as_bytes()
            .to_owned()
            .into_boxed_slice(),
    )?;

    // Tag should be present
    let tip_id = db.get_tag(MAIN_TAG)?.unwrap();
    let distance = db.get_block_info(tip_id.as_ref())?.chain_length();

    let mut vote_plan_files = HashMap::new();

    let block_iter = db.iter(tip_id.as_ref(), distance)?;

    for iter_res in block_iter {
        let block_bin = iter_res?;
        let mut codec = Codec::new(block_bin.as_ref());
        let block: Block = Block::deserialize(&mut codec).unwrap();

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

                let writer = vote_plan_files
                    .entry(certificate.vote_plan().clone())
                    .or_insert_with(|| {
                        let mut path = output_dir.to_path_buf();
                        path.push(format!("vote_plan_{}.csv", certificate.vote_plan()));
                        let file = std::fs::File::create(path).unwrap();
                        csv::Writer::from_writer(file)
                    });

                let choice = match certificate.payload() {
                    chain_impl_mockchain::vote::Payload::Public { choice } => choice.as_byte(),
                    chain_impl_mockchain::vote::Payload::Private { .. } => {
                        // zeroing data to enable private voting support
                        // (at least everying exception choice, since it is disabled by desing in private vote)
                        0u8
                    }
                };

                writer.serialize(Vote {
                    fragment_id: fragment_id.to_string(),
                    caster,
                    proposal: certificate.proposal_index(),
                    time: block.header().block_date().to_string(),
                    raw_fragment: hex::encode(tx.as_ref()),
                    choice,
                })?;
            }
        }
    }
    Ok(())
}
