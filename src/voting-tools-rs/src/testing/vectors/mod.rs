use cardano_serialization_lib::chain_crypto::{bech32, ed25519, AsymmetricKey, Ed25519};
pub use danger_rng::DangerRng;
use rand::SeedableRng;

mod danger_rng;

/// Generate a random cip_15 registration and associated signature
pub fn generate_cip_15(mut rng: DangerRng) {
    let voting_key = Ed25519::generate(&mut rng);
    let staking_key = Ed25519::generate(&mut rng);
    let payment_key = Ed25519::generate(&mut rng);
}

const METADATA_FROM_CIP_15: &str = include_str!("metadata");
const SIGNATURE_FROM_CIP_15: &str = include_str!("signature");

#[test]
fn cip_15_is_valid() {
    let rng = DangerRng::from_seed([0; 32]);
    let cip_15 = generate_cip_15(rng);

    panic!()
}

fn json_to_cbor_str(value: Value) -> String {

}
