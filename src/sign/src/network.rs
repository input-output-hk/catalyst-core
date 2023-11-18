//!
//! Test code
//! Example code on how to send a raw vote fragment
//!

use color_eyre::Result;

use reqwest::blocking::Client;
use reqwest::header::HeaderMap;

use reqwest::Url;
use serde::Deserialize as Deser;
use serde::Serialize as Ser;

use reqwest::header::{HeaderValue, CONTENT_TYPE};

/// Node responds with yay or nay and associated metadata such as fragment id hash
#[derive(Ser, Deser, Debug)]
pub struct NodeResponse {
    pub accepted: Vec<String>,
    pub rejected: Vec<Rejected>,
}

/// Vote fragment rejected
#[derive(Ser, Deser, Debug)]
pub struct Rejected {
    pub id: String,
    pub reason: String,
}

/// Vote fragment accepted
#[derive(Ser, Deser, Debug)]
pub struct Accepted {
    pub id: String,
}

/// Simple toy network network client for sending vote fragments
pub struct Network {
    pub client: Client,
    /// URL for posting a signed vote fragment
    /// e.g https://core.projectcatalyst.io/api/v0/message
    pub fragment_url: String,
}

impl Network {
    pub fn new(fragment_url: String) -> Self {
        Self {
            client: Client::new(),
            fragment_url,
        }
    }

    // Send single vote fragment to node
    pub fn send_fragment(
        &self,
        fragment: Vec<u8>,
    ) -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {
        Ok(self
            .client
            .post(Url::parse(&self.fragment_url)?)
            .headers(self.construct_headers())
            .body(fragment)
            .send()?)
    }

    /// construct headers for octet-stream
    fn construct_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
        headers
    }
}

mod tests {
    use crate::network::{Network, NodeResponse};
    use ed25519_dalek::Keypair;
    use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
    use rand_core::OsRng;

    use crate::fragment::{compose_encrypted_vote_part, generate_vote_fragment};
    use chain_vote::{Crs, ElectionPublicKey, MemberCommunicationKey, MemberState};

    fn create_election_pub_key(shared_string: String, mut rng: ChaCha20Rng) -> ElectionPublicKey {
        let h = Crs::from_hash(shared_string.as_bytes());
        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public()];
        let threshold = 1;
        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);
        let participants = vec![m1.public_key()];
        ElectionPublicKey::from_participants(&participants)
    }

    #[test]
    fn send_raw_fragment() {
        let client = Network::new("https://core.dev.projectcatalyst.io/api/v0/message".to_string());

        let mut csprng = OsRng;

        // User key for signing witness
        let keypair = Keypair::generate(&mut csprng);

        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        // vote plan id
        let vote_plan_id =
            "36ad42885189a0ac3438cdb57bc8ac7f6542e05a59d1f2e4d1d38194c9d4ac7b".to_owned();

        // election public key
        let ek = create_election_pub_key(vote_plan_id.clone(), rng.clone());

        // vote
        let vote = chain_vote::Vote::new(2, 1_usize).unwrap();

        let crs = chain_vote::Crs::from_hash(&hex::decode(vote_plan_id.as_bytes()).unwrap());

        let (ciphertexts, proof) = ek.encrypt_and_prove_vote(&mut rng, &crs, vote);
        let (proof, encrypted_vote) =
            compose_encrypted_vote_part(ciphertexts.clone(), proof).unwrap();

        // generate fragment
        let fragment_bytes = generate_vote_fragment(
            keypair,
            encrypted_vote,
            proof,
            5,
            &hex::decode(vote_plan_id.clone()).unwrap(),
            560,
            120,
        )
        .unwrap();

        let response = client.send_fragment(fragment_bytes).unwrap();

        let resp_json = response.json::<NodeResponse>().unwrap();

        println!("{:?}", resp_json);
    }
}
