use crate::accounting::account;
use crate::key::{deserialize_public_key, serialize_public_key};
use crate::transaction::WitnessAccountData;
use chain_core::property::WriteError;
use chain_core::{
    packer::Codec,
    property::{DeserializeFromSlice, ReadError, Serialize},
};
use chain_crypto::{Ed25519, PublicKey, Signature};

pub use account::{DelegationRatio, DelegationType, LedgerError, SpendingCounter};

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

impl Serialize for Identifier {
    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        serialize_public_key(&self.0, codec)
    }
}

impl DeserializeFromSlice for Identifier {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        deserialize_public_key(codec).map(Identifier)
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
}
