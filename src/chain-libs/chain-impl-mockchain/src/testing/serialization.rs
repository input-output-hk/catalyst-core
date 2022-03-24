use chain_core::{
    packer::Codec,
    property::{DeserializeFromSlice, Serialize},
};
use quickcheck::{Arbitrary, TestResult};

/// test that any arbitrary given object can serialize and deserialize
/// back into itself (i.e. it is a bijection,  or a one to one match
/// between the serialized bytes and the object)
pub fn serialization_bijection<T>(t: T) -> TestResult
where
    T: Arbitrary + Serialize + DeserializeFromSlice + Eq,
{
    let vec = match t.serialize_as_vec() {
        Err(error) => return TestResult::error(format!("serialization: {}", error)),
        Ok(v) => v,
    };
    let decoded_t = match T::deserialize_from_slice(&mut Codec::new(vec.as_slice())) {
        Err(error) => return TestResult::error(format!("deserialization: {:?}", error)),
        Ok(v) => v,
    };
    TestResult::from_bool(decoded_t == t)
}
