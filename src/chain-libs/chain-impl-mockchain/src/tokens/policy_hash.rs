use chain_core::mempack::{ReadBuf, ReadError, Readable};

pub const POLICY_HASH_SIZE: usize = 28;

/// blake2b_224 hash of a serialized minting policy
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PolicyHash([u8; POLICY_HASH_SIZE]);

impl AsRef<[u8]> for PolicyHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<[u8; POLICY_HASH_SIZE]> for PolicyHash {
    fn from(bytes: [u8; POLICY_HASH_SIZE]) -> Self {
        Self(bytes)
    }
}

impl TryFrom<&[u8]> for PolicyHash {
    type Error = ReadError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::read(&mut ReadBuf::from(value))
    }
}

impl Readable for PolicyHash {
    fn read(buf: &mut ReadBuf) -> Result<Self, ReadError> {
        let bytes = buf
            .get_slice(POLICY_HASH_SIZE)?
            .try_into()
            .unwrap_or_else(|_| panic!("already read {} bytes", POLICY_HASH_SIZE));
        Ok(Self(bytes))
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use quickcheck::TestResult;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for PolicyHash {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut bytes = [0u8; POLICY_HASH_SIZE];
            for i in &mut bytes {
                *i = Arbitrary::arbitrary(g);
            }
            Self(bytes)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn policy_hash_serialization_bijection(ph: PolicyHash) -> TestResult {
        let ph_got = ph.as_ref();
        let mut buf = ReadBuf::from(ph_got);
        let result = PolicyHash::read(&mut buf);
        let left = Ok(ph.clone());
        assert_eq!(left, result);
        assert_eq!(buf.get_slice_end(), &[]);
        TestResult::from_bool(left == result)
    }
}
