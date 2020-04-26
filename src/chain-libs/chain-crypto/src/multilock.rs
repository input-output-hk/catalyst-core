//! Similar to asymlock but takes a set of recipients instead of a unique receiver
//!
//! * ristretto-curve25519 for DH
//! * HKDF for KDF
//! * chacha20poly1305 for symmetric encryption algorithm
//!
//! the format is the concat of:
//!
//! * 1 byte of version: hardcoded to 1
//! * 1 byte of magic set to 0x12
//! * 1 byte of magic set to 0x34
//! * 1 byte from the number of participants
//! * ephemeral public key: 32 bytes
//! * recipient data (number of participants time) where each recipient is:
//!   * recipient public key
//!   * session key
//! * encrypted payload (cipher=chacha-poly1305)
//!   * authenticated tag
//!   * encrypted data
//!
//! the data encrypted with a ephemeral public key in prefix and
//! the poly1305 tag in suffix.
use cryptoxide::chacha20poly1305::ChaCha20Poly1305;
use cryptoxide::hkdf::hkdf_expand;
use cryptoxide::sha2;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use rand_core::{CryptoRng, RngCore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecryptionError {
    DataNoHeader,
    DataTooSmall,
    PointInvalid,
    TagMismatch,
    VersionUnknown,
    HeaderNotValid,
    PkNotParticipant,
}

fn shared_key_to_symmetric_key(app_level_info: &[u8], prk: &[u8]) -> ChaCha20Poly1305 {
    assert_eq!(prk.len(), 16);
    let mut symkey = [0u8; 16 + 12];
    hkdf_expand(sha2::Sha256::new(), prk, app_level_info, &mut symkey);
    ChaCha20Poly1305::new(&symkey[0..16], &symkey[16..], &[])
}

const HEADER_SIZE: usize = 4;
const EPHEMERAL_PK_SIZE: usize = 32;
const TAG_SIZE: usize = 16;
const RECIPIENT_KEY_SIZE: usize = 32;
const SESSION_KEY_SIZE: usize = 16;

// version 1 padding
const PAD1: u8 = 0x12;
const PAD2: u8 = 0x34;

fn recipient_start(ith: usize) -> usize {
    assert!(ith < 256);
    HEADER_SIZE + EPHEMERAL_PK_SIZE + ith * (RECIPIENT_KEY_SIZE + SESSION_KEY_SIZE)
}

fn recipient_public_key_nth(slice: &[u8], ith: usize) -> &[u8] {
    let start = recipient_start(ith);
    &slice[start..start + RECIPIENT_KEY_SIZE]
}

fn recipient_public_key_nth_point(
    slice: &[u8],
    ith: usize,
) -> Result<RistrettoPoint, DecryptionError> {
    let pk_slice = recipient_public_key_nth(slice, ith);
    let pk = CompressedRistretto::from_slice(pk_slice);
    pk.decompress().ok_or(DecryptionError::PointInvalid)
}

fn recipient_session_key_nth(slice: &[u8], ith: usize) -> &[u8] {
    let start = recipient_start(ith);
    &slice[start + RECIPIENT_KEY_SIZE..recipient_start(ith + 1)]
}

const fn prefix_size(participants: usize) -> usize {
    HEADER_SIZE + EPHEMERAL_PK_SIZE + participants * (SESSION_KEY_SIZE + RECIPIENT_KEY_SIZE)
}

const fn scheme_overhead(participants: usize) -> usize {
    // 32 bytes for each public keys and 16 bytes for each session keys + 16 bytes of tag
    prefix_size(participants) + TAG_SIZE
}

/// Encrypt data in an assymetric lock for multiple recipients
pub fn encrypt<R: RngCore + CryptoRng>(
    rng: &mut R,
    app_info: &[u8],
    receiver_pks: &[RistrettoPoint],
    data: &[u8],
) -> Vec<u8> {
    assert!(!receiver_pks.is_empty() && receiver_pks.len() < 256);
    // create a new ephemeral key and throw away the secret key keeping only the public key
    // and the shared key
    let r = Scalar::random(rng);
    let session_key = {
        let mut session_key = [0u8; 16];
        rng.fill_bytes(&mut session_key);
        session_key
    };

    let pk = RISTRETTO_BASEPOINT_POINT * r;

    // encrypt the data with the context
    let mut out = Vec::new();

    out.push(1);
    out.push(PAD1);
    out.push(PAD2);
    out.push(receiver_pks.len() as u8);

    // Copy the ephemeral key first
    out.extend_from_slice(pk.compress().as_bytes());

    for receiver_pk in receiver_pks {
        let shared_point = r * receiver_pk;
        out.extend_from_slice(&receiver_pk.compress().to_bytes());
        let receiver_shared = shared_point.compress().to_bytes();
        for (s1, s2) in session_key.iter().zip(receiver_shared.iter()) {
            out.push(s1 ^ s2)
        }
    }

    // Create a ChaCha20Poly1305 encryption context
    let mut context = shared_key_to_symmetric_key(app_info, &session_key);

    let mut payload = vec![0u8; TAG_SIZE + data.len()];

    let (tag, encrypted) = payload.split_at_mut(TAG_SIZE);
    context.encrypt(data, encrypted, tag);

    out.extend_from_slice(&payload);
    out
}

/// Decrypt data in the asymmetric lock. this is the dual of 'encrypt'.
/// The data should in the form:
///
/// ```text
///     EPHEMERAL_PUBLIC_KEY || SESSION_KEYS || ENCRYPTED-DATA || POLY1305-TAG
/// ```
///
/// # Return
///
/// Error if:
/// * header is invalid
/// * version is unknown
/// * data is too small
/// * any of the point is not in the first format
/// * tag don't match
/// Success otherwise
///
/// # Panics
///
/// If output 'out' is not of correct size
///
pub fn decrypt(
    app_info: &[u8],
    sk: &Scalar,
    data: &[u8],
    out: &mut [u8],
) -> Result<(), DecryptionError> {
    let participants = nb_participants(data)?;

    assert_eq!(data.len() - scheme_overhead(participants), out.len());

    let recipient_key = {
        let pk = RISTRETTO_BASEPOINT_POINT * sk;
        let pk_bytes = pk.compress().to_bytes();
        let mut found = None;
        for i in 0..participants {
            if recipient_public_key_nth(data, i) == pk_bytes {
                found = Some(recipient_session_key_nth(data, i))
            }
        }

        if let Some(k) = found {
            k
        } else {
            return Err(DecryptionError::PkNotParticipant);
        }
    };

    let pk_data = &data[4..36];
    let pk = CompressedRistretto::from_slice(pk_data);
    let shared = sk * pk.decompress().ok_or(DecryptionError::PointInvalid)?;
    let mut session_key = [0u8; 16];
    for (o, (x1, x2)) in session_key.iter_mut().zip(
        recipient_key
            .iter()
            .zip(shared.compress().to_bytes().iter()),
    ) {
        *o = x1 ^ x2
    }

    let start_payload = prefix_size(participants);
    let tag = &data[start_payload..start_payload + TAG_SIZE];
    let emsg = &data[start_payload + TAG_SIZE..];

    let mut context = shared_key_to_symmetric_key(app_info, &session_key);
    if !context.decrypt(emsg, out, tag) {
        return Err(DecryptionError::TagMismatch);
    }
    Ok(())
}

pub fn nb_participants(data: &[u8]) -> Result<usize, DecryptionError> {
    if data.len() < HEADER_SIZE {
        return Err(DecryptionError::DataTooSmall);
    }
    if data[0] != 1 {
        return Err(DecryptionError::VersionUnknown);
    }
    if data[1] != PAD1 || data[2] != PAD2 {
        return Err(DecryptionError::HeaderNotValid);
    }
    if data[3] == 0 {
        return Err(DecryptionError::HeaderNotValid);
    }
    let participants = data[3] as usize;

    if data.len() < scheme_overhead(participants) {
        return Err(DecryptionError::DataTooSmall);
    }

    Ok(participants)
}

pub fn participants(data: &[u8]) -> Result<Vec<RistrettoPoint>, DecryptionError> {
    let nb = nb_participants(data)?;
    let mut parts = Vec::new();

    for i in 0..nb {
        let point = recipient_public_key_nth_point(data, i)?;
        parts.push(point)
    }
    Ok(parts)
}

#[cfg(test)]
mod test {
    use super::*;
    use rand_core::OsRng;

    #[test]
    pub fn it_works() {
        let mut r = OsRng;

        let nb_participants = 3;

        let mut participants = Vec::with_capacity(nb_participants);
        for _ in 0..nb_participants {
            let sk_receiver = Scalar::random(&mut r);
            participants.push(sk_receiver)
        }

        let participant_pks = participants
            .iter()
            .map(|sk| RISTRETTO_BASEPOINT_POINT * sk)
            .collect::<Vec<_>>();

        let app_info = b"hello";
        let msg = b"message";

        let encrypted = encrypt(&mut r, app_info, &participant_pks, msg);
        for (i, sk) in participants.iter().enumerate() {
            let mut out = vec![0; msg.len()];
            //let pk = RISTRETTO_BASEPOINT_POINT * sk;
            decrypt(app_info, &sk, &encrypted, &mut out).unwrap();
            assert_eq!(out, msg, "cannot decrypt for participant {}", i);
        }
    }
}
