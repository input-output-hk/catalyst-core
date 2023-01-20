pub use danger_rng::DangerRng;

use crate::data::SignedRegistration;

/// CIP-15 test vectors
pub mod cip15;
mod danger_rng;


#[cfg(test)]
mod tests {
    use serde_json::Value;

    fn cip_15_test_vector() -> Value {
        serde_json::json!({
            "61284": {
                "1": "0036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a0",
                "2": "86870efc99c453a873a16492ce87738ec79a0ebd064379a62e2c9cf4e119219e",
                "3": "e0ae3a0a7aeda4aea522e74e4fe36759fca80789a613a58a4364f6ecef",
                "4": 1234,
            },
            "61285": {
                "1": "6c2312cd49067ecf0920df7e067199c55b3faef4ec0bce1bd2cfb99793972478c45876af2bc271ac759c5ce40ace5a398b9fdb0e359f3c333fe856648804780e",
            }
        })
    }

    #[test]
    fn cip_15_test_vector_is_valid() {
        let metadata = serde_json::json!({
            "1": "0036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a0",
            "2": "86870efc99c453a873a16492ce87738ec79a0ebd064379a62e2c9cf4e119219e",
            "3": "e0ae3a0a7aeda4aea522e74e4fe36759fca80789a613a58a4364f6ecef",
            "4": 1234,
        });

        let signature = serde_json::json!({
            "1": "6c2312cd49067ecf0920df7e067199c55b3faef4ec0bce1bd2cfb99793972478c45876af2bc271ac759c5ce40ace5a398b9fdb0e359f3c333fe856648804780e",
        });
    }
}
