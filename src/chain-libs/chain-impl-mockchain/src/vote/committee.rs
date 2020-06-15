use chain_core::{
    mempack::{ReadBuf, ReadError, Readable},
    property,
};
use chain_crypto::{Ed25519, PublicKey};
use std::{
    convert::TryFrom,
    fmt::{self, Debug, Display},
    str::FromStr,
};
use thiserror::Error;

/// committee identifier
///
/// this value is used to identify a committee member on chain
/// as well as to use as input for the vote casting payload.
///
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommitteeId([u8; CommitteeId::COMMITTEE_ID_SIZE]);

impl CommitteeId {
    pub const COMMITTEE_ID_SIZE: usize = 32;

    /// returns the identifier encoded in hexadecimal string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// read the identifier from the hexadecimal string
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0; Self::COMMITTEE_ID_SIZE];
        hex::decode_to_slice(s, &mut bytes)?;
        Ok(CommitteeId(bytes))
    }

    pub fn public_key(&self) -> PublicKey<Ed25519> {
        self.clone().into()
    }
}

/* Conversion ************************************************************** */

impl From<PublicKey<Ed25519>> for CommitteeId {
    fn from(key: PublicKey<Ed25519>) -> Self {
        Self::try_from(key.as_ref()).unwrap()
    }
}

impl From<[u8; Self::COMMITTEE_ID_SIZE]> for CommitteeId {
    fn from(id: [u8; Self::COMMITTEE_ID_SIZE]) -> Self {
        Self(id)
    }
}

impl From<CommitteeId> for [u8; CommitteeId::COMMITTEE_ID_SIZE] {
    fn from(id: CommitteeId) -> Self {
        id.0
    }
}

impl From<CommitteeId> for PublicKey<Ed25519> {
    fn from(id: CommitteeId) -> Self {
        PublicKey::from_binary(id.0.as_ref())
            .expect("CommitteeId should be a valid Ed25519 public key")
    }
}

/// error that can be received when converting a slice into a
/// [`CommitteeId`].
///
/// [`CommitteeId`]: ./struct.CommitteeId.html
///
#[derive(Debug, Error)]
#[error("Not enough bytes, expected {expected} but received {received}")]
pub struct TryFromCommitteeIdError {
    expected: usize,
    received: usize,
}

impl<'a> TryFrom<&'a [u8]> for CommitteeId {
    type Error = TryFromCommitteeIdError;
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() != Self::COMMITTEE_ID_SIZE {
            Err(TryFromCommitteeIdError {
                expected: Self::COMMITTEE_ID_SIZE,
                received: value.len(),
            })
        } else {
            let mut committee_id = Self([0; Self::COMMITTEE_ID_SIZE]);
            committee_id.0.copy_from_slice(value);
            Ok(committee_id)
        }
    }
}

/* AsRef ******************************************************************* */

impl AsRef<[u8]> for CommitteeId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/* Display ***************************************************************** */

impl Display for CommitteeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl Debug for CommitteeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CommitteeId").field(&self.to_hex()).finish()
    }
}

/* FromStr ***************************************************************** */

impl FromStr for CommitteeId {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

/* Ser/De ****************************************************************** */

impl property::Serialize for CommitteeId {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        writer.write_all(self.as_ref())
    }
}

impl Readable for CommitteeId {
    fn read<'a>(reader: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let slice = reader.get_slice(Self::COMMITTEE_ID_SIZE)?;
        Self::try_from(slice).map_err(|err| ReadError::StructureInvalid(err.to_string()))
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    #[cfg(test)]
    use chain_core::property::Serialize as _;
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for CommitteeId {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut bytes = [0; Self::COMMITTEE_ID_SIZE];
            g.fill_bytes(&mut bytes);
            Self(bytes)
        }
    }

    #[quickcheck]
    fn to_from_hex(committee_id: CommitteeId) -> bool {
        let s = committee_id.to_hex();
        let d = CommitteeId::from_hex(&s).expect("decode hexadecimal committee id");

        committee_id == d
    }

    #[quickcheck]
    fn display_parse(committee_id: CommitteeId) -> bool {
        let s = committee_id.to_string();
        let d = s.parse().expect("decode hexadecimal committee id");

        committee_id == d
    }

    #[quickcheck]
    fn serialize_readable(committee_id: CommitteeId) -> bool {
        let b_got = committee_id.serialize_as_vec().unwrap();
        let mut buf = ReadBuf::from(b_got.as_ref());
        let result = CommitteeId::read(&mut buf).expect("decode the committee ID");
        committee_id == result
    }
}
