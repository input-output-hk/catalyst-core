use bech32::{self, FromBase32};
use chain_addr::{Address, Kind};
use chain_crypto::{AsymmetricPublicKey, Ed25519, PublicKey};
use chain_impl_mockchain::account;

use chain_addr::{AddressReadable, Discrimination};
use chain_core::{packer::Codec, property::DeserializeFromSlice};

use chain_impl_mockchain::{
    block::Block, chaintypes::HeaderId, fragment::Fragment, transaction::InputEnum,
};

use jormungandr_lib::interfaces::AccountIdentifier;

const MAIN_TAG: &str = "HEAD";

use std::path::Path;

use crate::offline::Vote;

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

    #[error("account parameter '{addr}' isn't a valid address or publickey")]
    NotRecognized { addr: String },
    #[error("account parameter '{addr}' isn't an account address, found: '{kind}'")]
    AddressNotAccount { addr: String, kind: String },
}

#[derive(Debug, Clone)]
pub struct AccountId {
    account: account::Identifier,
}

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

pub fn bytes_to_pub_key<K: AsymmetricPublicKey>(bytes: &[u8]) -> Result<String, Error> {
    pub use chain_crypto::bech32::Bech32 as _;
    let public: chain_crypto::PublicKey<K> = chain_crypto::PublicKey::from_binary(bytes).unwrap();
    Ok(public.to_bech32_str())
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chain_crypto::Ed25519;

    use crate::find::find_vote;

    use super::{bytes_to_pub_key, AccountId};

    #[test]
    #[ignore]
    fn test_account_parser() {
        let voting_key =
            "f895a6a7f44dd15f7700c60456c93793b1241fdd1c77bbb6cd3fc8a4d24c8c1b".to_string();

        let decoded = hex::decode(voting_key).unwrap();
        let pub_key = bytes_to_pub_key::<Ed25519>(&decoded).unwrap();

        let account = AccountId::try_from_str(&pub_key).unwrap();

        println!("account {:?}", account.to_url_arg());
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
