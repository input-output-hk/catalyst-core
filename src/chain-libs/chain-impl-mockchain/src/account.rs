use crate::accounting::account;
use crate::key::{deserialize_public_key, serialize_public_key};
use crate::transaction::WitnessAccountData;
use chain_core::{
    mempack::{ReadBuf, ReadError, Readable},
    property,
};
use chain_crypto::{AsymmetricPublicKey, Ed25519, PublicKey, Signature};

pub use account::{DelegationRatio, DelegationType, LedgerError, SpendingCounter};
use chain_ser::deser::Deserialize;
use chain_ser::packer::Codec;
use std::io::Error;

pub type AccountAlg = Ed25519;

pub type Witness = Signature<WitnessAccountData, AccountAlg>;

/// Account Identifier (also used as Public Key)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier(PublicKey<AccountAlg>);

impl From<PublicKey<AccountAlg>> for Identifier {
    fn from(pk: PublicKey<AccountAlg>) -> Self {
        Identifier(pk)
    }
}

impl From<Identifier> for PublicKey<AccountAlg> {
    fn from(i: Identifier) -> Self {
        i.0
    }
}

impl AsRef<PublicKey<AccountAlg>> for Identifier {
    fn as_ref(&self) -> &PublicKey<AccountAlg> {
        &self.0
    }
}


fn pack_identifier<W: std::io::Write>(identifier: &Identifier, codec: &mut Codec<W>) -> Result<(), std::io::Error> {
    serialize_public_key(&identifier.0, codec)
}

fn unpack_identifier<R: std::io::BufRead>(codec: &mut Codec<R>) -> Result<Identifier, std::io::Error> {
    let bytes =
        codec.get_bytes(<AccountAlg as AsymmetricPublicKey>::PUBLIC_KEY_SIZE as usize)?;
    let mut bytes_buff = ReadBuf::from(&bytes);
    match Identifier::read(&mut bytes_buff) {
        Ok(identifier) => Ok(identifier),
        Err(e) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Error reading Identifier: {}", e),
        )),
    }
}


impl Readable for Identifier {
    fn read<'a>(reader: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        deserialize_public_key(reader).map(Identifier)
    }
}

/// The public ledger of all accounts associated with their current state
pub type Ledger = account::Ledger<Identifier, ()>;

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod test {
    use super::*;
    use chain_crypto::{Ed25519, KeyPair};
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Identifier {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let kp: KeyPair<Ed25519> = Arbitrary::arbitrary(g);
            Identifier::from(kp.into_keys().1)
        }
    }

    #[quickcheck]
    pub fn identifier_pack_unpack_bijection<G: Gen>(g: &mut G) {
        let kp: KeyPair<Ed25519> = Arbitrary::arbitrary(g);
    }
}
