use bech32::{self, FromBase32};
use chain_addr::{Address, AddressReadable, Kind};
use chain_crypto::{Ed25519, PublicKey};
use chain_impl_mockchain::account;

use chain_addr::Discrimination;
use chain_core::{packer::Codec, property::DeserializeFromSlice};

use chain_impl_mockchain::{
    block::Block, chaintypes::HeaderId, fragment::Fragment, transaction::InputEnum,
};

use tracing::{error, info};

use jormungandr_lib::interfaces::AccountIdentifier;

const MAIN_TAG: &str = "HEAD";

use std::{
    collections::{HashMap, HashSet},
    error,
    fs::{read_to_string, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

use crate::offline::{extract_fragments_from_storage, Vote};

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

    #[error("Corrupted key")]
    CorruptedKey,

    #[error("Unable to extract Tally fragment")]
    CorruptedFragments,

    #[error("account parameter '{addr}' isn't a valid address or publickey")]
    NotRecognized { addr: String },
    #[error("account parameter '{addr}' isn't an account address, found: '{kind}'")]
    AddressNotAccount { addr: String, kind: String },
}

#[derive(Debug, Clone)]
pub struct AccountId {
    account: account::Identifier,
}

/// Generate account identifier from ED25519 Key
fn id_from_pub(pk: PublicKey<Ed25519>) -> account::Identifier {
    account::Identifier::from(pk)
}

impl AccountId {
    // accept either an address with the account kind
    // or a ed25519 publickey
    pub fn try_from_str(src: &str) -> Result<Self, Error> {
        if let Ok((_, data, _variant)) = bech32::decode(src) {
            let dat = Vec::from_base32(&data).unwrap();
            if let Ok(addr) = Address::from_bytes(&dat) {
                match addr.kind() {
                    Kind::Account(pk) => Ok(Self {
                        account: id_from_pub(pk.clone()),
                    }),
                    _ => Err(Error::AddressNotAccount {
                        addr: src.to_string(),
                        kind: format!("{:?}", addr.kind()),
                    }),
                }
            } else if let Ok(pk) = PublicKey::from_binary(&dat) {
                Ok(Self {
                    account: id_from_pub(pk),
                })
            } else {
                Err(Error::NotRecognized {
                    addr: src.to_string(),
                })
            }
        } else {
            Err(Error::NotRecognized {
                addr: src.to_string(),
            })
        }
    }

    // account id is encoded in hexadecimal in url argument
    pub fn to_url_arg(&self) -> String {
        hex::encode(self.account.as_ref().as_ref())
    }
}

/// Did I vote?
/// Iterate through all vote cast fragments and match the given voters pub key to confirm vote "went through".
///
pub fn find_vote(jormungandr_database: &Path, voting_key: String) -> Result<Vec<Vote>, Error> {
    let db = chain_storage::BlockStore::file(
        jormungandr_database,
        HeaderId::zero_hash()
            .as_bytes()
            .to_owned()
            .into_boxed_slice(),
    )?;

    let decoded_voting_key = match hex::decode(voting_key) {
        Ok(decoded_key) => decoded_key,
        Err(err) => {
            error!("err decoding key {}", err);
            return Err(Error::CorruptedKey);
        }
    };

    let voting_key: PublicKey<Ed25519> = match PublicKey::from_binary(&decoded_voting_key) {
        Ok(voting_key) => voting_key,
        Err(err) => {
            error!("err parsing pub key from bin {}", err);
            return Err(Error::CorruptedKey);
        }
    };

    let caster_address = Address(Discrimination::Production, Kind::Account(voting_key));

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

/// Collect all voting keys in ca and 0x format and write to files
pub fn all_voters(
    jormungandr_database: &Path,
) -> Result<(HashSet<std::string::String>, HashSet<std::string::String>), Box<dyn error::Error>> {
    let fragments = extract_fragments_from_storage(jormungandr_database)?;

    let mut unique_voters_ca = HashSet::new();
    let mut unique_voters_0x = HashSet::new();

    for fragment in fragments {
        if let Fragment::VoteCast(tx) = fragment.clone() {
            let input = tx.as_slice().inputs().iter().next().unwrap().to_enum();
            let caster = if let InputEnum::AccountInput(account_id, _value) = input {
                AccountIdentifier::from(account_id).into_address(Discrimination::Production, "ca")
            } else {
                error!("Corrupted fragment {:?}", fragment);
                continue;
            };

            unique_voters_ca.insert(caster.to_string().clone());

            let voting_key_61824_format = AddressReadable::from_string("ca", &caster.to_string())
                .expect("infallible")
                .to_address();

            let voting_key = voting_key_61824_format
                .public_key()
                .expect("infallible")
                .to_string();
            unique_voters_0x.insert(voting_key);
        }
    }

    info!("unique voters ca {:?}", unique_voters_ca.len());
    info!("unique voters 0x {:?}", unique_voters_0x.len());

    Ok((unique_voters_ca, unique_voters_0x))
}

/// convert keys from ca to 0x and vice versa
pub fn convert_key_formats(voting_key: String) -> Result<String, Box<dyn error::Error>> {
    if voting_key.starts_with("ca") {
        let voting_key_61824_format = AddressReadable::from_string("ca", &voting_key)?.to_address();

        let voting_key = voting_key_61824_format
            .public_key()
            .expect("addr to pub key is infallible")
            .to_string();

        Ok(voting_key)
    } else {
        // we need to convert this to our internal key representation
        let decoded_voting_key = hex::decode(voting_key)?;
        let voting_key: PublicKey<Ed25519> = PublicKey::from_binary(&decoded_voting_key)?;
        let addr = Address(Discrimination::Production, Kind::Single(voting_key.clone()));
        let addr_readable = AddressReadable::from_address("ca", &addr);

        Ok(addr_readable.to_string())
    }
}

/// read voter keys from file
pub fn read_lines(filename: &str) -> Result<Vec<String>, Box<dyn error::Error>> {
    let mut result = Vec::new();

    for line in read_to_string(filename)?.lines() {
        result.push(line.to_string())
    }

    Ok(result)
}

/// check key history of multiple keys and write metadata to file
pub fn batch_key_check(
    jormungandr_database: &Path,
    key_file: String,
) -> Result<(), Box<dyn error::Error>> {
    let mut flagged_keys = HashMap::new();

    let keys = read_lines(&key_file)?;

    for key in keys {
        let voting_key_61824_format = AddressReadable::from_string("ca", &key)
            .expect("infallible")
            .to_address();

        let voting_key = voting_key_61824_format
            .public_key()
            .expect("infallible")
            .to_string();

        let votes = find_vote(jormungandr_database, voting_key)?;

        flagged_keys.insert(key.clone(), votes.clone());

        info!("Inserted: key: {} vote: {:?}", key, votes);
    }

    let flagged_file = PathBuf::from("/tmp/inspect").with_extension("flag_keys.json");

    let file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(flagged_file.clone())?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &flagged_keys)?;

    info!("flagged keys and metadata saved here {:?}", flagged_file);

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use chain_addr::{Address, AddressReadable, Discrimination, Kind};
    use chain_crypto::{Ed25519, PublicKey};

    use crate::find::find_vote;

    use super::convert_key_formats;

    #[test]
    fn test_key_conversion() {
        let voting_key_0x =
            "f895a6a7f44dd15f7700c60456c93793b1241fdd1c77bbb6cd3fc8a4d24c8c1b".to_string();

        let converted_key = convert_key_formats(voting_key_0x.clone()).unwrap();

        let voting_key_ca =
            "ca1q0uftf4873xazhmhqrrqg4kfx7fmzfqlm5w80wake5lu3fxjfjxpk6wv3f7".to_string();

        assert_eq!(converted_key, voting_key_ca,);

        assert_eq!(convert_key_formats(voting_key_ca).unwrap(), voting_key_0x);
    }

    #[test]
    #[ignore]
    fn test_account_parser() {
        // voting key as per CIP-36: 61284 format
        // random key not from any fund
        let voting_key =
            "f895a6a7f44dd15f7700c60456c93793b1241fdd1c77bbb6cd3fc8a4d24c8c1b".to_string();

        // we need to convert this to our internal key representation
        let decoded_voting_key = hex::decode(voting_key).unwrap();
        let voting_key: PublicKey<Ed25519> = PublicKey::from_binary(&decoded_voting_key).unwrap();
        let addr = Address(Discrimination::Production, Kind::Single(voting_key.clone()));
        let addr_readable = AddressReadable::from_address("ca", &addr);

        println!("{:?}", addr_readable);
        assert_eq!(
            "ca1q0uftf4873xazhmhqrrqg4kfx7fmzfqlm5w80wake5lu3fxjfjxpk6wv3f7".to_string(),
            addr_readable.to_string()
        );
    }

    #[test]
    #[ignore]
    fn test_key_transformation() {
        // test internal key representation transform to 61284 representation
        // 61284 representation as seen by voter in TX Metadata

        // internal address representation address from fund9
        let voting_key =
            "ca1qhjmpfwz2rmck46t3vtjsw7vd3mf9ae0ckqfpa9q5gmzf97j35dg2wapv8u".to_string();

        let voting_key_61824_format = AddressReadable::from_string("ca", &voting_key)
            .unwrap()
            .to_address();

        let voting_key = voting_key_61824_format.public_key().unwrap().to_string();

        assert_eq!(
            voting_key,
            "e5b0a5c250f78b574b8b17283bcc6c7692f72fc58090f4a0a2362497d28d1a85"
        );
    }

    #[test]
    #[ignore]
    fn test_find_vote() {
        let path = PathBuf::from("/tmp/fund9-leader-1/persist/leader-1");

        // ca1qhjmpfwz2rmck46t3vtjsw7vd3mf9ae0ckqfpa9q5gmzf97j35dg2wapv8u = e5b0a5c250f78b574b8b17283bcc6c7692f72fc58090f4a0a2362497d28d1a85
        let voting_key =
            "e5b0a5c250f78b574b8b17283bcc6c7692f72fc58090f4a0a2362497d28d1a85".to_string();

        let votes = find_vote(&path, voting_key).unwrap();

        assert_eq!(votes.len(), 286);
    }
}
