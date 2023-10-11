//! Generate Fragments based upon specification
//! Reference specfication for more context in relation to constants outlined in this file.

use chain_ser::packer::Codec;

use chain_vote::{Ciphertext, ProofOfCorrectVote};
use ed25519_dalek::{ed25519::signature::Signature, *};
use std::error;

/// Payload type = 2
/// %x02 ENCRYPTED-VOTE PROOF-VOTE ; Private payload
const ENCRYPTED_PAYLOAD: u8 = 2;

/// VoteCast tag
const VOTE_CAST_TAG: u8 = 11;

/// INPUT-ACCOUNT = %xff VALUE UNTAG-ACCOUNT-ID
const INPUT_ACCOUNT: u8 = 255;

/// Block epoch + slot
/// This is redundant as time checks have been removed
const EPOCH: u32 = 0;
const SLOT: u32 = 0;

/// Only 1 input (subsequently 1 witness), no output
/// VoteCast TX should have only 1 input, 0 output and 1 witness (signature).
const INPUT: u8 = 1;
const OUTPUT: u8 = 0;

/// Nonce
const NONCE: u32 = 0;

/// Type = 2
/// utxo witness scheme
/// ED25519 Signature (64 bytes)
const WITNESS_SCHEME: u8 = 2;

/// Padding
const PADDING: u8 = 0;

/// Values in inputs: redundant for voting
const VALUE: u64 = 0;

/// Padding and Tag are 1 byte each; size must be added to the fragment size
const PADDING_AND_TAG_SIZE: u32 = 2;

/// Generate vote fragment in bytes
pub fn generate_vote_fragment(
    keypair: Keypair,
    encrypted_vote: Vec<u8>,
    proof: Vec<u8>,
    proposal: u8,
    vote_plan_id: &[u8],
) -> Result<Vec<u8>, Box<dyn error::Error>> {
    let mut vote_cast = Codec::new(Vec::new());

    vote_cast.put_bytes(vote_plan_id)?;
    vote_cast.put_u8(proposal)?;
    vote_cast.put_u8(ENCRYPTED_PAYLOAD)?;
    vote_cast.put_bytes(&encrypted_vote)?;
    vote_cast.put_bytes(&proof)?;

    let data_to_sign = vote_cast.into_inner().clone();

    let (inputs, witness) = compose_inputs_and_witnesses(keypair, data_to_sign.clone())?;

    let mut vote_cast = Codec::new(Vec::new());
    vote_cast.put_bytes(&data_to_sign)?;
    vote_cast.put_bytes(&inputs)?;
    vote_cast.put_bytes(&witness)?;

    let data = vote_cast.into_inner();

    // prepend msg with size of fragment msg
    let mut vote_cast = Codec::new(Vec::new());
    vote_cast.put_be_u32(data.len() as u32 + PADDING_AND_TAG_SIZE)?;
    vote_cast.put_u8(PADDING)?;
    vote_cast.put_u8(VOTE_CAST_TAG)?;
    vote_cast.put_bytes(&data.as_slice())?;

    Ok(vote_cast.into_inner())
}

/// Generate Inputs-Outputs-Witnesses in bytes
fn compose_inputs_and_witnesses(
    keypair: Keypair,
    data_to_sign: Vec<u8>,
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn error::Error>> {
    let mut inputs = Codec::new(Vec::new());

    inputs.put_be_u32(EPOCH)?;
    inputs.put_be_u32(SLOT)?;
    inputs.put_u8(INPUT)?;
    inputs.put_u8(OUTPUT)?;

    inputs.put_u8(INPUT_ACCOUNT)?;
    inputs.put_be_u64(VALUE)?;
    inputs.put_bytes(keypair.public.as_bytes())?;
    let inputs = inputs.into_inner().clone();

    let mut tx_data_to_sign = Codec::new(Vec::new());
    tx_data_to_sign.put_bytes(&data_to_sign.clone())?;
    tx_data_to_sign.put_bytes(&inputs.clone())?;

    let signature = keypair.sign(&tx_data_to_sign.into_inner());

    let mut witness = Codec::new(Vec::new());
    witness.put_u8(WITNESS_SCHEME)?;
    witness.put_be_u32(NONCE)?;
    witness.put_bytes(signature.as_bytes())?;
    let witnesses = witness.into_inner();

    Ok((inputs, witnesses))
}

/// compose encrypted vote and proof in bytes
pub fn compose_encrypted_vote_part(
    ciphertexts: Vec<Ciphertext>,
    proof: ProofOfCorrectVote,
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn error::Error>> {
    let mut encrypted_vote = Codec::new(Vec::new());

    let size_element = ciphertexts.iter().len();
    for cipher in ciphertexts.iter() {
        encrypted_vote.put_bytes(&cipher.to_bytes())?;
    }

    let encrypted_bytes = encrypted_vote.into_inner();

    // prepend with SIZE-ELEMENT-8BIT
    let mut encrypted_vote = Codec::new(Vec::new());
    encrypted_vote.put_u8(size_element as u8)?;
    encrypted_vote.put_bytes(&encrypted_bytes.as_slice())?;

    let mut proof_bytes = Codec::new(Vec::new());

    for announcement in proof.announcments_group_elements() {
        proof_bytes.put_bytes(&announcement.to_bytes())?;
    }

    for cipher in proof.ds().into_iter() {
        proof_bytes.put_bytes(&cipher.to_bytes())?;
    }

    for response in proof.zwvs().into_iter() {
        proof_bytes.put_bytes(&response.to_bytes())?;
    }

    proof_bytes.put_bytes(&proof.r().as_bytes())?;

    // prepend with SIZE-ELEMENT-8BIT
    let mut proof_vote = Codec::new(Vec::new());
    proof_vote.put_u8(proof.len() as u8)?;
    proof_vote.put_bytes(proof_bytes.into_inner().as_slice())?;

    let mut proof = Codec::new(Vec::new());

    proof.put_bytes(&proof_vote.into_inner())?;

    Ok((proof.into_inner(), encrypted_vote.into_inner()))
}

#[cfg(test)]
mod tests {

    use chain_addr::{AddressReadable, Discrimination};

    use chain_impl_mockchain::{fragment::Fragment, transaction::InputEnum};
    use chain_ser::{deser::DeserializeFromSlice, packer::Codec};

    use ed25519_dalek::Keypair;
    use rand_core::OsRng;

    use chain_vote::{
        Ciphertext, Crs, ElectionPublicKey, MemberCommunicationKey, MemberState, ProofOfCorrectVote,
    };

    use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};

    use crate::fragment::{compose_encrypted_vote_part, generate_vote_fragment};
    use jormungandr_lib::interfaces::AccountIdentifier;

    #[test]
    fn test_fragment_generation() {
        let mut csprng = OsRng;

        // User key for signing witness
        let keypair = Keypair::generate(&mut csprng);

        let pk = keypair.public.as_bytes().clone();

        println!("Secret key: {}", hex::encode(keypair.secret.as_bytes()));
        println!("Public key: {}", hex::encode(keypair.public.as_bytes()));

        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        // vote plan id
        let vote_plan_id =
            "36ad42885189a0ac3438cdb57bc8ac7f6542e05a59d1f2e4d1d38194c9d4ac7b".to_owned();

        // election public key
        let ek = create_election_pub_key(vote_plan_id.clone(), rng.clone());

        println!("election public key {:?}", hex::encode(ek.to_bytes()));

        // vote
        let vote = chain_vote::Vote::new(2, 1 as usize).unwrap();

        let crs = chain_vote::Crs::from_hash(vote_plan_id.as_bytes());

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
        )
        .unwrap();

        println!(
            "generated fragment: {:?} size:{:?}",
            hex::encode(fragment_bytes.clone()),
            fragment_bytes.len()
        );

        let fragment = Fragment::deserialize_from_slice(&mut Codec::new(&fragment_bytes)).unwrap();

        if let Fragment::VoteCast(tx) = fragment.clone() {
            let _fragment_id = fragment.hash();

            let input = tx.as_slice().inputs().iter().next().unwrap().to_enum();
            let caster = if let InputEnum::AccountInput(account_id, _value) = input {
                AccountIdentifier::from(account_id).into_address(Discrimination::Production, "ca")
            } else {
                panic!("unhandled input ");
            };
            let certificate = tx.as_slice().payload().into_payload();

            let voting_key_61824_format = AddressReadable::from_string("ca", &caster.to_string())
                .unwrap()
                .to_address();

            let voting_key = voting_key_61824_format.public_key().unwrap().to_string();

            assert_eq!(voting_key, hex::encode(pk));
            assert_eq!(certificate.proposal_index(), 5);
            assert_eq!(certificate.vote_plan().to_string(), vote_plan_id);
        }
    }

    fn create_election_pub_key(shared_string: String, mut rng: ChaCha20Rng) -> ElectionPublicKey {
        let h = Crs::from_hash(shared_string.as_bytes());
        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public()];
        let threshold = 1;
        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);
        let participants = vec![m1.public_key()];
        let ek = ElectionPublicKey::from_participants(&participants);
        ek
    }

    #[test]
    fn generate_keys_from_bytes() {
        let pk = hex::decode(
            "ac247e6cbc2106a8858d67a9b6aa9fc6105a2f42abfd8d269f4096488b7e5d81".to_string(),
        )
        .unwrap();

        let mut sk = hex::decode(
            "40cc7f02e04324b63a4db949854d5f24c9041a2bebe9b42064ff868071d1d72d".to_string(),
        )
        .unwrap();

        sk.extend(pk.clone());
        let keys = sk.clone();
        let keypair: Keypair = Keypair::from_bytes(&keys).unwrap();

        assert_eq!(hex::encode(keypair.public.as_bytes()), hex::encode(pk));

        println!("Secret key: {}", hex::encode(keypair.secret.as_bytes()));
        println!("Public key: {}", hex::encode(keypair.public.as_bytes()));
    }

    #[test]
    fn test_encrypted_vote_generation() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        // vote plan id
        let vote_plan_id =
            "36ad42885189a0ac3438cdb57bc8ac7f6542e05a59d1f2e4d1d38194c9d4ac7b".to_owned();

        let shared_string = vote_plan_id.to_owned();

        // election public key
        let ek = create_election_pub_key(shared_string, rng.clone());

        let vote = chain_vote::Vote::new(2, 1 as usize).unwrap();
        let crs = chain_vote::Crs::from_hash(vote_plan_id.as_bytes());

        let (ciphertexts, proof) = ek.encrypt_and_prove_vote(&mut rng, &crs, vote);
        let (proof, mut enc_vote) =
            compose_encrypted_vote_part(ciphertexts.clone(), proof).unwrap();

        // remove size element, size element is 2 meaning there two ciphertexts
        enc_vote.remove(0);
        // each ciphertext consists of two 32 byte group elements
        let (cipher_a, cipher_b) = enc_vote.split_at(64);

        let _cipher_a = Ciphertext::from_bytes(cipher_a).unwrap();
        let _cipher_b = Ciphertext::from_bytes(cipher_b).unwrap();

        let mut msg = Codec::new(proof.as_slice());

        let p = ProofOfCorrectVote::from_buffer(&mut msg).unwrap();

        assert_eq!(p.len(), 1);
    }
}
