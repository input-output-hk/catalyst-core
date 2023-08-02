use bech32::{self, Error as Bech32Error, FromBase32};
use bech32::{ToBase32, Variant};

use chain_crypto::testing::TestCryptoRng;
use chain_crypto::{Ed25519, SecretKey};
use chain_vote::committee::MemberSecretKey;
use chain_vote::tally::batch_decrypt;
use chain_vote::{EncryptedTally, MemberPublicKey, Tally};

use chain_vote::TallyDecryptShare;

use base64::{engine::general_purpose, Engine as _};

use color_eyre::Result;
use rand_core::SeedableRng;

/// A Bech32_encoded address consists of 3 parts: A Human-Readable Part (HRP) + Separator + Data:
const HRP_PK: &str = "ristretto255_memberpk";
const HRP_SK: &str = "ristretto255_membersk";

///
/// Get member's secret share
///
pub fn get_members_secret_share(
    key: String,
) -> Result<MemberSecretKey, Box<dyn std::error::Error>> {
    let (_hrp, data, _variant) = bech32::decode(&key).map_err(Bech32Error::from)?;

    let bytes = Vec::<u8>::from_base32(&data).map_err(Bech32Error::from)?;

    Ok(MemberSecretKey::from_bytes(&bytes).ok_or("member secret key from bytes")?)
}
///
/// Get member's public share
///
pub fn get_members_public_share(
    key: String,
) -> Result<MemberPublicKey, Box<dyn std::error::Error>> {
    let (_hrp, data, _variant) = bech32::decode(&key).map_err(Bech32Error::from)?;

    let bytes = Vec::<u8>::from_base32(&data).map_err(Bech32Error::from)?;

    Ok(MemberPublicKey::from_bytes(&bytes).ok_or("member public key from bytes")?)
}

///
/// Extract decyrpt shares for publication
///
pub fn extract_decrypt_shares(
    encrypted_tally: EncryptedTally,
    committee_priv_keys: Vec<MemberSecretKey>,
) -> Vec<TallyDecryptShare> {
    let mut rng = TestCryptoRng::seed_from_u64(0);

    let mut shares = vec![];

    for member_sk in committee_priv_keys {
        // Given a single committee member's `secret_key`, returns a partial decryption (share) of the `EncryptedTally`
        shares.push(encrypted_tally.partial_decrypt(&mut rng, &member_sk));
    }

    shares
}

///
/// Parse private committee keys from Bech32 representation
///
pub fn parse_private_committee_keys(
    committee_keys: Vec<String>,
) -> Result<(Vec<MemberPublicKey>, Vec<MemberSecretKey>), Box<dyn std::error::Error>> {
    let mut priv_keys = vec![];
    let mut pub_keys = vec![];

    for member_sk in committee_keys {
        let secret_key = get_members_secret_share(member_sk)?;
        priv_keys.push(secret_key.clone());
        pub_keys.push(secret_key.to_public());
    }

    Ok((pub_keys, priv_keys))
}

///
/// Parse public committee keys from Bech32 representation
///
pub fn parse_public_committee_keys(
    committee_keys: Vec<String>,
) -> Result<Vec<MemberPublicKey>, Box<dyn std::error::Error>> {
    let mut pub_keys = vec![];
    for member_pk in committee_keys {
        pub_keys.push(get_members_public_share(member_pk)?);
    }
    Ok(pub_keys)
}

///
/// Load encrypted tally
///
pub fn load_encrypted_tally(et: String) -> Result<EncryptedTally, Box<dyn std::error::Error>> {
    EncryptedTally::from_base_64(et)
}

///
/// Load decrypt shares
///
pub fn load_decrypt_shares(
    shares: Vec<String>,
) -> Result<Vec<TallyDecryptShare>, Box<dyn std::error::Error>> {
    let mut decrypt_shares = vec![];

    for share in shares {
        let bytes = general_purpose::STANDARD.decode(share)?;
        let tally_decrypt_share =
            TallyDecryptShare::from_bytes(&bytes).ok_or("TallyDecryptShare from bytes error")?;
        decrypt_shares.push(tally_decrypt_share)
    }

    Ok(decrypt_shares)
}

///
/// Encode decrypt shares to base64 for publication
///
pub fn encode_decrypt_shares(decrypt_shares: Vec<TallyDecryptShare>) -> Vec<String> {
    let mut shares = vec![];
    for share in decrypt_shares {
        shares.push(general_purpose::STANDARD.encode(share.to_bytes()))
    }
    shares
}

/// Encode committee secret keys to Bech32
pub fn encode_secret_keys(
    committee_secret_keys: Vec<SecretKey<Ed25519>>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut keys = vec![];

    for key in committee_secret_keys {
        keys.push(bech32::encode(
            HRP_SK,
            key.leak_secret().to_base32(),
            Variant::Bech32,
        )?)
    }
    Ok(keys)
}

///
/// Encode public shares to bech32
///
pub fn encode_public_keys(
    committee_pub_keys: Vec<MemberPublicKey>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut keys = vec![];

    for key in committee_pub_keys {
        keys.push(bech32::encode(
            HRP_PK,
            key.to_bytes().to_base32(),
            Variant::Bech32,
        )?)
    }

    Ok(keys)
}

///
/// Decrypt tally with secret keys
/// Internal use only
pub fn decrypt_tally_with_secret_keys(
    encrypted_tally: EncryptedTally,
    committee_priv_keys: Vec<MemberSecretKey>,
) -> Result<Vec<Tally>, Box<dyn std::error::Error>> {
    let mut rng = TestCryptoRng::seed_from_u64(0);

    let mut public_keys = vec![];

    let mut shares = vec![];

    for member_sk in committee_priv_keys {
        // Given a single committee member's `secret_key`, returns a partial decryption (share) of the `EncryptedTally`
        public_keys.push(member_sk.to_public());
        shares.push(encrypted_tally.partial_decrypt(&mut rng, &member_sk));
    }

    let validated_tally = encrypted_tally.validate_partial_decryptions(&public_keys, &shares)?;

    Ok(batch_decrypt([validated_tally])?)
}

#[cfg(test)]
mod tests {

    use rand_chacha::ChaCha20Rng;

    use chain_vote::{tally::batch_decrypt, EncryptedTally};
    use chain_vote::{Ballot, Crs, ElectionPublicKey, MemberCommunicationKey, MemberState, Vote};
    use rand_core::{CryptoRng, RngCore, SeedableRng};

    use crate::tally::{
        encode_decrypt_shares, encode_public_keys, extract_decrypt_shares, load_decrypt_shares,
        load_encrypted_tally, parse_public_committee_keys,
    };

    fn get_encrypted_ballot<R: RngCore + CryptoRng>(
        rng: &mut R,
        pk: &ElectionPublicKey,
        crs: &Crs,
        vote: Vote,
    ) -> Ballot {
        let (enc, proof) = pk.encrypt_and_prove_vote(rng, crs, vote);
        Ballot::try_from_vote_and_proof(enc, &proof, crs, pk).unwrap()
    }

    #[test]
    pub fn test_validation_logic() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let shared_string = b"common reference string (CRS)".to_owned();
        let h = Crs::from_hash(&shared_string);

        let alice = MemberCommunicationKey::new(&mut rng);

        let bob = MemberCommunicationKey::new(&mut rng);

        let charlie = MemberCommunicationKey::new(&mut rng);

        let threshold = 1;

        let alice = MemberState::new(&mut rng, threshold, &h, &[alice.to_public()], 0);
        let bob = MemberState::new(&mut rng, threshold, &h, &[bob.to_public()], 0);
        let charlie = MemberState::new(&mut rng, threshold, &h, &[charlie.to_public()], 0);

        let committee_public_keys =
            vec![alice.public_key(), bob.public_key(), charlie.public_key()];
        let committee_secret_keys = vec![
            alice.member_secret_key(),
            bob.member_secret_key(),
            charlie.member_secret_key(),
        ];

        let ek = ElectionPublicKey::from_participants(&committee_public_keys);

        println!("encrypting vote");

        let vote_options = 2;
        let e1 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 0).unwrap());
        let e2 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 1).unwrap());
        let e3 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 0).unwrap());

        println!("tallying");

        let mut encrypted_tally = EncryptedTally::new(vote_options, ek.clone(), h.clone());
        encrypted_tally.add(&e1, 1);
        encrypted_tally.add(&e2, 3);
        encrypted_tally.add(&e3, 4);

        // Ingredients to publish for community validation (decrypt shares, pub keys, encrypted tally)

        //
        // decrypt shares
        //

        let shares = extract_decrypt_shares(encrypted_tally.clone(), committee_secret_keys);

        let published_shares = encode_decrypt_shares(shares.clone());

        let loaded_shares = load_decrypt_shares(published_shares).unwrap();

        assert_eq!(shares, loaded_shares);

        //
        // encrypted tally
        //

        let published_encrypted_tally = encrypted_tally.to_base64();

        let loaded_encrypted_tally = load_encrypted_tally(published_encrypted_tally).unwrap();

        assert_eq!(encrypted_tally, loaded_encrypted_tally);

        //
        // pub keys
        //

        let pub_keys = encode_public_keys(committee_public_keys.clone()).unwrap();

        let pub_keys = parse_public_committee_keys(pub_keys).unwrap();

        assert_eq!(pub_keys, committee_public_keys);

        //
        // public decryption
        // (decrypt shares, pub keys, encrypted tally)
        //

        let validated_tally = encrypted_tally
            .validate_partial_decryptions(&pub_keys, &shares)
            .unwrap();

        let tally = &batch_decrypt([validated_tally]).unwrap()[0];

        assert_eq!(tally.votes, vec![5, 3]);

        assert!(tally.verify(&encrypted_tally, &committee_public_keys, &shares));

        println!("results from decryption: {:?}", tally);
    }
}
