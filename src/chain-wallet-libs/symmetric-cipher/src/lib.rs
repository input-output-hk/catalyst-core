use cryptoxide::chacha20poly1305::ChaCha20Poly1305;
use cryptoxide::hmac::Hmac;
use cryptoxide::pbkdf2::pbkdf2;
use cryptoxide::sha2::Sha512;
use std::convert::TryInto;
use thiserror::Error;
use zeroize::Zeroizing;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid payload protocol")]
    InvalidProtocol,
    #[error("expected data to be a multiple of 64 bytes")]
    InvalidDataLength,
    #[error("encrypted data should not be null")]
    EmptyPayload,
    #[error("missing data")]
    MalformedInput,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("wrong password")]
    AuthenticationFailed,
}

struct View<T: AsRef<[u8]>>(pub T);

const ITERS: u32 = 12983;
const PROTOCOL_SIZE: usize = 1;
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;

pub fn encrypt<G: rand::Rng + rand::CryptoRng>(
    password: impl AsRef<[u8]>,
    data: impl AsRef<[u8]>,
    mut random: G,
) -> Result<Box<[u8]>, Error> {
    if data.as_ref().is_empty() {
        return Err(Error::EmptyPayload);
    }

    if data.as_ref().len() % 64 != 0 {
        return Err(Error::InvalidDataLength);
    }

    let aad = [];

    let salt = {
        let mut salt = [0u8; SALT_SIZE];
        random.fill(&mut salt);
        salt
    };

    let nonce = {
        let mut nonce = [0u8; NONCE_SIZE];
        random.fill(&mut nonce);
        nonce
    };

    let symmetric_key = derive_symmetric_key(password, salt);

    let mut chacha20 = ChaCha20Poly1305::new(&*symmetric_key, &nonce, &aad);

    let (ciphertext, tag) = {
        let mut ciphertext = vec![0u8; data.as_ref().len()];
        let mut tag = [0u8; 16];
        chacha20.encrypt(data.as_ref(), &mut ciphertext, &mut tag);

        (ciphertext.into_boxed_slice(), tag)
    };

    let mut buffer =
        vec![0u8; PROTOCOL_SIZE + SALT_SIZE + NONCE_SIZE + ciphertext.len() + TAG_SIZE];

    let parts: [&[u8]; 5] = [&[1u8], &salt, &nonce, &ciphertext, &tag];

    let mut low = 0;

    for part in parts.iter() {
        buffer[low..low + part.len()].copy_from_slice(part);
        low += part.len();
    }

    Ok(buffer.into_boxed_slice())
}

pub fn decrypt<T: AsRef<[u8]>>(password: impl AsRef<[u8]>, data: T) -> Result<Box<[u8]>, Error> {
    let data = View::new(data)?;

    let aad = [];

    if data.protocol() != 0x1 {
        return Err(Error::InvalidProtocol);
    }

    let key = derive_symmetric_key(password, data.salt().try_into().unwrap());

    let mut chacha20 = ChaCha20Poly1305::new(&*key, data.nonce(), &aad);

    let mut plaintext = vec![0u8; data.encrypted_data().len()];

    if chacha20.decrypt(data.encrypted_data(), &mut plaintext, data.tag()) {
        Ok(plaintext.into_boxed_slice())
    } else {
        Err(Error::AuthenticationFailed)
    }
}

impl<T: AsRef<[u8]>> View<T> {
    fn new(inner: T) -> Result<View<T>, Error> {
        if inner.as_ref().len() <= PROTOCOL_SIZE + SALT_SIZE + NONCE_SIZE + TAG_SIZE {
            Err(Error::MalformedInput)
        } else {
            let data = Self(inner);

            if data.encrypted_data().is_empty() {
                Err(Error::EmptyPayload)
            } else if data.encrypted_data().len() % 64 != 0 {
                Err(Error::InvalidDataLength)
            } else {
                Ok(data)
            }
        }
    }

    fn protocol(&self) -> u8 {
        self.0.as_ref()[0]
    }

    fn salt(&self) -> &[u8] {
        &self.0.as_ref()[PROTOCOL_SIZE..PROTOCOL_SIZE + SALT_SIZE]
    }

    fn nonce(&self) -> &[u8] {
        &self.0.as_ref()[PROTOCOL_SIZE + SALT_SIZE..PROTOCOL_SIZE + SALT_SIZE + NONCE_SIZE]
    }

    fn encrypted_data(&self) -> &[u8] {
        let data_len = self
            .0
            .as_ref()
            .len()
            .checked_sub(PROTOCOL_SIZE + SALT_SIZE + NONCE_SIZE + TAG_SIZE)
            .unwrap();

        let starting_pos = PROTOCOL_SIZE + SALT_SIZE + NONCE_SIZE;
        &self.0.as_ref()[starting_pos..starting_pos + data_len]
    }

    fn tag(&self) -> &[u8] {
        let start = self.0.as_ref().len().checked_sub(TAG_SIZE).unwrap();
        &self.0.as_ref()[start..]
    }
}

fn derive_symmetric_key(password: impl AsRef<[u8]>, salt: [u8; SALT_SIZE]) -> Zeroizing<[u8; 32]> {
    let mut symmetric_key = [0u8; 32];

    let mut mac = Hmac::new(Sha512::new(), password.as_ref());
    pbkdf2(&mut mac, &salt[..], ITERS, &mut symmetric_key);

    Zeroizing::new(symmetric_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn get_random_gen() -> rand_chacha::ChaChaRng {
        rand_chacha::ChaChaRng::seed_from_u64(33)
    }

    #[test]
    fn encrypt_decrypt() {
        let mut bytes: Vec<u8> = vec![];
        bytes.extend([1u8; 64].iter());
        bytes.extend([2u8; 64].iter());
        bytes.extend([3u8; 64].iter());

        let password = [1u8, 2, 3, 4];

        let slice = encrypt(&password, &bytes[..], get_random_gen()).unwrap();

        assert_eq!(&decrypt(&password, slice).unwrap()[..], &bytes[..]);
    }

    #[test]
    fn encrypt_non_64_bytes() {
        let bytes = [0u8; 65];
        let password = [1u8, 2, 3, 4];

        assert!(encrypt(&password, &bytes[..], get_random_gen()).is_err())
    }

    #[test]
    fn encrypt_decrypt_empty_payload() {
        let bytes = [];
        let password = [1u8, 2, 3, 4];

        assert!(encrypt(&password, &bytes[..], get_random_gen()).is_err())
    }

    #[test]
    fn test_vectors_account_two_utxo() {
        let account = [
            200u8, 101, 150, 194, 209, 32, 136, 133, 219, 31, 227, 101, 132, 6, 170, 15, 124, 199,
            184, 225, 60, 54, 47, 228, 106, 109, 178, 119, 252, 80, 100, 88, 62, 72, 117, 136, 201,
            138, 108, 54, 226, 231, 68, 92, 10, 221, 54, 248, 63, 23, 28, 181, 204, 253, 129, 85,
            9, 209, 156, 211, 142, 203, 10, 243,
        ];

        let key1 = [
            48, 21, 89, 204, 178, 212, 204, 126, 158, 84, 166, 245, 90, 128, 150, 11, 182, 145,
            183, 177, 64, 149, 73, 239, 134, 149, 169, 46, 164, 26, 111, 79, 64, 82, 49, 168, 6,
            194, 231, 185, 208, 219, 48, 225, 94, 224, 204, 31, 38, 28, 27, 159, 150, 21, 99, 107,
            72, 189, 137, 254, 123, 230, 234, 31,
        ];

        let key2 = [
            168, 182, 189, 240, 128, 199, 79, 188, 49, 51, 126, 222, 75, 102, 146, 194, 235, 237,
            126, 52, 175, 109, 152, 183, 187, 205, 71, 140, 240, 123, 13, 94, 217, 63, 126, 157,
            74, 163, 175, 222, 50, 26, 225, 171, 182, 27, 131, 68, 194, 67, 201, 208, 180, 7, 203,
            248, 145, 125, 182, 223, 44, 101, 61, 234,
        ];

        let password = [1u8, 2, 3, 4];

        let mut bytes = [0u8; 64 * 3];

        bytes[0..64].copy_from_slice(&account);
        bytes[64..2 * 64].copy_from_slice(&key1);
        bytes[2 * 64..3 * 64].copy_from_slice(&key2);

        let slice = encrypt(&password, &bytes[..], get_random_gen()).unwrap();
        assert_eq!(&decrypt(&password, slice).unwrap()[..], &bytes[..]);
    }

    #[test]
    fn wrong_password() {
        let mut bytes: Vec<u8> = vec![];
        bytes.extend([1u8; 64].iter());
        bytes.extend([2u8; 64].iter());
        bytes.extend([3u8; 64].iter());

        let password = [1u8, 2, 3, 4];

        let slice = encrypt(&password, &bytes[..], get_random_gen()).unwrap();

        let password = [5u8, 6, 7, 8];
        assert!(matches!(
            decrypt(&password, slice),
            Err(Error::AuthenticationFailed)
        ));
    }
}
