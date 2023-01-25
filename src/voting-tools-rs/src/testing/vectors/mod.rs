/// CIP-15 test vectors
pub mod cip15;

#[cfg(test)]
mod tests {
    use crate::{data::Registration, Signature};

    use serde_json::{from_value, json, Value};

    fn cip_15_test_vector() -> (Value, Value) {
        let reg = json!({
            "1": "0036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a0",
            "2": "86870efc99c453a873a16492ce87738ec79a0ebd064379a62e2c9cf4e119219e",
            "3": "e0ae3a0a7aeda4aea522e74e4fe36759fca80789a613a58a4364f6ecef",
            "4": 1234,
        });

        let sig = json!({
            "1": "6c2312cd49067ecf0920df7e067199c55b3faef4ec0bce1bd2cfb99793972478c45876af2bc271ac759c5ce40ace5a398b9fdb0e359f3c333fe856648804780e",
        });

        (reg, sig)
    }

    #[test]
    fn can_deserialize_cip15_test_vector() {
        let (reg, sig) = cip_15_test_vector();
        let _reg: Registration = from_value(reg).unwrap();
        let _sig: Signature = from_value(sig).unwrap();
    }
}
