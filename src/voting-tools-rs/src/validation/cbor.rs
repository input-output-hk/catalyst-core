use ciborium::value::Value;

pub(crate) fn cbor_to_bytes(cbor: &Value) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![];
    ciborium::ser::into_writer(cbor, &mut bytes).unwrap();
    bytes
}

#[cfg(test)]
mod tests {
    use super::cbor_to_bytes;

    use ciborium::{
        cbor,
        value::{Error, Value},
    };

    fn check(value: Result<Value, Error>, hex_bytes: &str) {
        let expected = hex::decode(hex_bytes).unwrap();
        let actual = cbor_to_bytes(&value.unwrap());

        assert_eq!(actual, expected);
    }

    // test cases generated from cyber chef
    #[test]
    fn cbor_to_bytes_works() {
        check(cbor!({ "hello" => "world" }), "a16568656c6c6f65776f726c64");
        check(cbor!({ "1" => "world" }), "a1613165776f726c64");
        check(cbor!({ 1 => "world" }), "a10165776f726c64");
    }
}
