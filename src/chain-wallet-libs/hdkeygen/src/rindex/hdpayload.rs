//! HD Payload
//!
//! The HD Payload is an Address attribute stored along the address
//! in encrypted form.
//!
//! This use chacha20poly1305 to auth-encrypt a BIP39 derivation
//! path, which is then stored in the address. The owner of the
//! symmetric key used to encrypt, can then decrypt the address
//! payload and find the derivation path associated with it.
//!
use chain_path_derivation::{AnyScheme, Derivation, DerivationPath};
use cryptoxide::{chacha20poly1305::ChaCha20Poly1305, hmac::Hmac, pbkdf2::pbkdf2, sha2::Sha512};
use ed25519_bip32::XPub;
use thiserror::Error;

const NONCE: &[u8] = b"serokellfore";
const SALT: &[u8] = b"address-hashing";
const TAG_LEN: usize = 16;
pub const HDKEY_SIZE: usize = 32;
/// This is the max size we accept to try to decrypt a HDPayload.
/// This is due to avoid trying to decrypt content that are way beyond
/// reasonable size.
pub const MAX_PAYLOAD_SIZE: usize = 48;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Cannot decrypt the Legacy Address Payload")]
    CannotDecrypt,
    #[error("Invalid Payload, expecting at least 16 bytes")]
    NotEnoughEncryptedData,
    /// this relates to the issue that addresses with the payload data
    /// can have an infinite length (as long as it fits in the max block size
    /// and max transaction size).
    #[error("the Legacy Address Payload is too large ({0} bytes), limited to {1} bytes max.")]
    PayloadIsTooLarge(usize, usize),
}

const CBOR_INDEFINITE_ARRAY: u8 = 0x9F;
const CBOR_BREAK: u8 = 0xFF;

#[cfg(test)]
const CBOR_MAX_INLINE_ENCODING: u32 = 23;
#[cfg(test)]
const CBOR_PAYLOAD_LENGTH_U8: u8 = 24;
#[cfg(test)]
const CBOR_PAYLOAD_LENGTH_U16: u8 = 25;
#[cfg(test)]
const CBOR_PAYLOAD_LENGTH_U32: u8 = 26;

#[cfg(test)]
fn encode_derivation(buf: &mut Vec<u8>, derivation: Derivation) {
    let value: u32 = *derivation;

    if value <= CBOR_MAX_INLINE_ENCODING {
        buf.push(value as u8);
    } else if value < 0x1_00 {
        buf.push(CBOR_PAYLOAD_LENGTH_U8);
        buf.push(value as u8);
    } else if value < 0x1_00_00 {
        buf.push(CBOR_PAYLOAD_LENGTH_U16);
        buf.push(((value & 0xFF_00) >> 8) as u8);
        buf.push((value & 0x00_FF) as u8);
    } else {
        buf.push(CBOR_PAYLOAD_LENGTH_U32);
        buf.push(((value & 0xFF_00_00_00) >> 24) as u8);
        buf.push(((value & 0x00_FF_00_00) >> 16) as u8);
        buf.push(((value & 0x00_00_FF_00) >> 8) as u8);
        buf.push((value & 0x00_00_00_FF) as u8);
    }
}

fn cursor_read(reader: &mut &[u8]) -> Option<u8> {
    use std::io::Read as _;
    let mut b = [0];
    let sz = reader.read(&mut b).ok()?;
    if sz == 1 {
        Some(b[0])
    } else {
        None
    }
}

fn decode_derivation(reader: &mut &[u8]) -> Option<Derivation> {
    let b: u8 = cursor_read(reader)?;
    let v = match b {
        0x00..=0x17 => b as u32,
        0x18 => cursor_read(reader)? as u32,
        0x19 => {
            let b1 = cursor_read(reader)? as u32;
            let b2 = cursor_read(reader)? as u32;
            b1 << 8 | b2
        }
        0x1a => {
            let b1 = cursor_read(reader)? as u32;
            let b2 = cursor_read(reader)? as u32;
            let b3 = cursor_read(reader)? as u32;
            let b4 = cursor_read(reader)? as u32;
            b1 << 24 | b2 << 16 | b3 << 8 | b4
        }
        _ => return None,
    };

    Some(Derivation::from(v))
}

#[cfg(test)]
fn encode_derivation_path<S>(derivation_path: &DerivationPath<S>) -> Vec<u8> {
    let mut buf = Vec::with_capacity(32);

    buf.push(CBOR_INDEFINITE_ARRAY);
    for derivation in derivation_path.iter().copied() {
        encode_derivation(&mut buf, derivation);
    }
    buf.push(CBOR_BREAK);

    buf
}

pub fn decode_derivation_path(buf: &[u8]) -> Option<DerivationPath<AnyScheme>> {
    let mut cursor = buf; // std::io::Cursor::new(buf);
    let mut dp = DerivationPath::new();

    if cursor_read(&mut cursor)? != CBOR_INDEFINITE_ARRAY {
        return None;
    }

    loop {
        let derivation = decode_derivation(&mut cursor)?;
        dp = dp.append_unchecked(derivation);

        if cursor.len() <= 1 {
            if cursor_read(&mut cursor)? != CBOR_BREAK {
                return None;
            } else {
                break;
            }
        }
    }

    Some(dp)
}

/// The key to encrypt and decrypt HD payload
#[derive(Clone, zeroize::ZeroizeOnDrop)]
pub struct HdKey([u8; HDKEY_SIZE]);
impl HdKey {
    /// Create a new `HDKey` from an extended public key
    pub fn new(root_pub: &XPub) -> Self {
        let mut mac = Hmac::new(Sha512::new(), root_pub.as_ref());
        let mut result = [0; HDKEY_SIZE];
        let iters = 500;
        pbkdf2(&mut mac, SALT, iters, &mut result);
        HdKey(result)
    }

    #[cfg(test)]
    pub fn encrypt(&self, input: &[u8]) -> Vec<u8> {
        let mut ctx = ChaCha20Poly1305::new(&self.0, NONCE, &[]);

        let len = input.len();

        let mut out: Vec<u8> = vec![0; len];
        let mut tag = [0; TAG_LEN];

        ctx.encrypt(input, &mut out[0..len], &mut tag);
        out.extend_from_slice(&tag[..]);
        out
    }

    pub fn decrypt(&self, input: &[u8]) -> Result<Vec<u8>, Error> {
        if input.len() <= TAG_LEN {
            return Err(Error::NotEnoughEncryptedData);
        };
        let len = input.len() - TAG_LEN;
        if len >= MAX_PAYLOAD_SIZE {
            return Err(Error::PayloadIsTooLarge(len, MAX_PAYLOAD_SIZE));
        }

        let mut ctx = ChaCha20Poly1305::new(&self.0, NONCE, &[]);

        let mut out: Vec<u8> = vec![0; len];

        if ctx.decrypt(&input[..len], &mut out[..], &input[len..]) {
            Ok(out)
        } else {
            Err(Error::CannotDecrypt)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_bip32::XPrv;

    const SEED_SIZE: usize = 64;

    fn generate_from_daedalus_seed(bytes: &[u8]) -> XPrv {
        use cryptoxide::mac::Mac;

        let mut mac = Hmac::new(Sha512::new(), bytes);

        let mut iter = 1;

        loop {
            let s = format!("Root Seed Chain {}", iter);
            mac.reset();
            mac.input(s.as_bytes());
            let mut block = [0u8; 64];
            mac.raw_result(&mut block);

            let mut sk = [0; 32];
            sk.clone_from_slice(&block.as_ref()[0..32]);
            let mut cc = [0; 32];
            cc.clone_from_slice(&block.as_ref()[32..64]);
            let xprv = XPrv::from_nonextended_force(&sk, &cc);

            // check if we find a good candidate
            if xprv.as_ref()[31] & 0x20 == 0 {
                return xprv;
            }

            iter += 1;
        }
    }

    #[test]
    fn encrypt() {
        let bytes = vec![42u8; MAX_PAYLOAD_SIZE - 1];
        let sk = generate_from_daedalus_seed(&[0; SEED_SIZE]);
        let pk = sk.public();

        let key = HdKey::new(&pk);
        let payload = key.encrypt(&bytes);
        assert_eq!(bytes, key.decrypt(&payload).unwrap())
    }

    #[test]
    fn decrypt_too_small() {
        const TOO_SMALL_PAYLOAD: usize = TAG_LEN - 1;
        let bytes = vec![42u8; TOO_SMALL_PAYLOAD];
        let sk = generate_from_daedalus_seed(&[0; SEED_SIZE]);
        let pk = sk.public();

        let key = HdKey::new(&pk);
        match key.decrypt(&bytes).unwrap_err() {
            Error::NotEnoughEncryptedData => {}
            err => unreachable!("expecting Error::NotEnoughEncryptedData but got {:#?}", err),
        }
    }
    #[test]
    fn decrypt_too_large() {
        const TOO_LARGE_PAYLOAD: usize = 2 * MAX_PAYLOAD_SIZE;
        let bytes = vec![42u8; TOO_LARGE_PAYLOAD];
        let sk = generate_from_daedalus_seed(&[0; SEED_SIZE]);
        let pk = sk.public();

        let key = HdKey::new(&pk);
        match key.decrypt(&bytes).unwrap_err() {
            Error::PayloadIsTooLarge(len, _too_large) => {
                assert_eq!(len, TOO_LARGE_PAYLOAD - TAG_LEN)
            }
            err => unreachable!(
                "expecting Error::PayloadIsTooLarge({}) but got {:#?}",
                TOO_LARGE_PAYLOAD - TAG_LEN,
                err
            ),
        }
    }

    #[test]
    fn path_cbor_encoding() {
        let path = derivation_path(&[0.into(), 1.into(), 2.into()]);
        let cbor = encode_derivation_path(&path);
        dbg!(&cbor);
        let expected = decode_derivation_path(&cbor);
        assert_eq!(Some(path), expected);
    }

    #[test]
    fn hdpayload() {
        let path = derivation_path(&[0.into(), 1.into(), 2.into()]);
        let path = encode_derivation_path(&path);
        let sk = generate_from_daedalus_seed(&[0; SEED_SIZE]);
        let pk = sk.public();

        let key = HdKey::new(&pk);
        let payload = key.encrypt(&path);
        assert_eq!(path, key.decrypt(&payload).unwrap())
    }

    #[test]
    fn unit1() {
        let key = HdKey([0u8; 32]);
        let dat = [0x9f, 0x00, 0x01, 0x0ff];
        let expected = [
            0xda, 0xac, 0x4a, 0x55, 0xfc, 0xa7, 0x48, 0xf3, 0x2f, 0xfa, 0xf4, 0x9e, 0x2b, 0x41,
            0xab, 0x86, 0xf3, 0x54, 0xdb, 0x96,
        ];
        let got = key.encrypt(&dat[..]);
        assert_eq!(&expected[..], &got[..])
    }

    #[test]
    fn unit2() {
        let path = derivation_path(&[0.into(), 1.into()]);
        let expected = [0x9f, 0x00, 0x01, 0x0ff];
        let cbor = encode_derivation_path(&path);
        assert_eq!(&expected[..], &cbor[..])
    }

    struct GoldenTest {
        xprv_key: [u8; ed25519_bip32::XPRV_SIZE],
        hdkey: [u8; HDKEY_SIZE],
        payload: &'static [u8],
        addressing: [Derivation; 2],
    }

    const GOLDEN_TESTS: &[GoldenTest] = &[
        GoldenTest {
            xprv_key: [
                32, 15, 90, 64, 107, 113, 208, 132, 181, 199, 158, 192, 82, 246, 119, 189, 80, 23,
                31, 95, 219, 198, 94, 39, 18, 166, 174, 186, 139, 177, 243, 82, 202, 175, 171, 241,
                217, 208, 101, 229, 20, 60, 84, 114, 214, 1, 73, 40, 25, 142, 239, 22, 239, 146,
                66, 82, 121, 206, 22, 120, 24, 45, 126, 66, 208, 108, 114, 200, 223, 219, 60, 98,
                75, 118, 2, 56, 104, 230, 68, 215, 229, 31, 241, 136, 165, 71, 176, 231, 189, 125,
                179, 211, 163, 66, 186, 210,
            ],
            hdkey: [
                96, 3, 72, 241, 97, 26, 53, 38, 110, 107, 149, 105, 139, 250, 203, 125, 73, 152,
                12, 195, 158, 54, 84, 69, 99, 239, 234, 122, 177, 179, 59, 200,
            ],
            payload: &[
                0x33, 0x1c, 0xd6, 0xc3, 0x02, 0x5d, 0x59, 0xa1, 0x6a, 0x5f, 0x82, 0x9e, 0xd7, 0xf2,
                0x4c, 0xf8, 0x74, 0xf3, 0xab, 0x50,
            ],
            addressing: [Derivation::new(0), Derivation::new(0)],
        },
        GoldenTest {
            xprv_key: [
                32, 15, 90, 64, 107, 113, 208, 132, 181, 199, 158, 192, 82, 246, 119, 189, 80, 23,
                31, 95, 219, 198, 94, 39, 18, 166, 174, 186, 139, 177, 243, 82, 202, 175, 171, 241,
                217, 208, 101, 229, 20, 60, 84, 114, 214, 1, 73, 40, 25, 142, 239, 22, 239, 146,
                66, 82, 121, 206, 22, 120, 24, 45, 126, 66, 208, 108, 114, 200, 223, 219, 60, 98,
                75, 118, 2, 56, 104, 230, 68, 215, 229, 31, 241, 136, 165, 71, 176, 231, 189, 125,
                179, 211, 163, 66, 186, 210,
            ],
            hdkey: [
                96, 3, 72, 241, 97, 26, 53, 38, 110, 107, 149, 105, 139, 250, 203, 125, 73, 152,
                12, 195, 158, 54, 84, 69, 99, 239, 234, 122, 177, 179, 59, 200,
            ],
            payload: &[
                0x33, 0x06, 0x56, 0x3c, 0x02, 0xd0, 0x2f, 0x38, 0x1e, 0x78, 0xdf, 0x84, 0x04, 0xc3,
                0x50, 0x56, 0x76, 0xd5, 0x5e, 0x45, 0x71, 0x93, 0xe7, 0x4a, 0x34, 0xb6, 0x90, 0xec,
            ],
            addressing: [Derivation::new(0x8000_0000), Derivation::new(0x8000_0000)],
        },
    ];

    fn derivation_path(d: &[Derivation]) -> DerivationPath<AnyScheme> {
        let mut dp = DerivationPath::new();

        for d in d {
            dp = dp.append_unchecked(*d);
        }

        dp
    }

    fn run_golden_test(golden_test: &GoldenTest) {
        let xprv = XPrv::from_bytes_verified(golden_test.xprv_key).unwrap();
        let hdkey = HdKey(golden_test.hdkey);
        let payload = Vec::from(golden_test.payload);
        let path = derivation_path(&golden_test.addressing[..]);
        let path = encode_derivation_path(&path);

        let our_hdkey = HdKey::new(&xprv.public());
        assert_eq!(hdkey.0, our_hdkey.0);

        let our_payload = hdkey.encrypt(&path);
        assert_eq!(payload, our_payload);

        let our_path = hdkey.decrypt(&payload).unwrap();
        assert_eq!(path, our_path);
    }

    #[test]
    fn golden_tests() {
        for golden_test in GOLDEN_TESTS {
            run_golden_test(golden_test)
        }
    }
}

#[cfg(test)]
#[cfg(feature = "with-bench")]
mod bench {
    use hdpayload::{self, *};
    use hdwallet;
    use test;

    #[bench]
    fn decrypt_fail(b: &mut test::Bencher) {
        let path = Path::new(vec![0, 1]);
        let seed = hdwallet::Seed::from_bytes([0; hdwallet::SEED_SIZE]);
        let sk = hdwallet::XPrv::generate_from_seed(&seed);
        let pk = sk.public();

        let key = HDKey::new(&pk);
        let payload = key.encrypt_path(&path);

        let seed = hdwallet::Seed::from_bytes([1; hdwallet::SEED_SIZE]);
        let sk = hdwallet::XPrv::generate_from_seed(&seed);
        let pk = sk.public();
        let key = HDKey::new(&pk);
        b.iter(|| {
            let _ = key.decrypt(&payload);
        })
    }

    #[bench]
    fn decrypt_ok(b: &mut test::Bencher) {
        let path = Path::new(vec![0, 1]);
        let seed = hdwallet::Seed::from_bytes([0; hdwallet::SEED_SIZE]);
        let sk = hdwallet::XPrv::generate_from_seed(&seed);
        let pk = sk.public();

        let key = HDKey::new(&pk);
        let payload = key.encrypt_path(&path);

        b.iter(|| {
            let _ = key.decrypt(&payload);
        })
    }

    #[bench]
    fn decrypt_with_cbor(b: &mut test::Bencher) {
        let path = Path::new(vec![0, 1]);
        let seed = hdwallet::Seed::from_bytes([0; hdwallet::SEED_SIZE]);
        let sk = hdwallet::XPrv::generate_from_seed(&seed);
        let pk = sk.public();

        let key = HDKey::new(&pk);
        let payload = key.encrypt_path(&path);

        b.iter(|| {
            let _ = key.decrypt_path(&payload);
        })
    }
}
