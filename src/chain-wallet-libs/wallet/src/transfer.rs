use chain_ser::packer::Codec;
use cryptoxide::chacha20poly1305::ChaCha20Poly1305;
use cryptoxide::hmac::Hmac;
use cryptoxide::pbkdf2::pbkdf2;
use cryptoxide::sha2::Sha512;
use std::convert::TryInto;
use std::marker::PhantomData;
use thiserror::Error;
use zeroize::Zeroize;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid payload protocol")]
    InvalidProtocol,
    #[error("expected data to be a multiple of 64 bytes")]
    InvalidDataLength,
    #[error("encrypted data should not be null")]
    EmptyPayload,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

const ITERS: u32 = 12983;
const PROTOCOL_SIZE: usize = 1;
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;

pub struct TransferSlice(pub Box<[u8]>);

struct TransferSliceBuilder<T>(Codec<Vec<u8>>, PhantomData<T>);

pub fn encrypt(password: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> Result<TransferSlice, Error> {
    if data.as_ref().is_empty() {
        return Err(Error::EmptyPayload);
    }

    let aad = [];

    let salt = {
        let mut salt = [0u8; SALT_SIZE];
        getrandom::getrandom(&mut salt).expect("Failed to generate random salt");
        salt
    };

    let nonce = {
        let mut nonce = [0u8; NONCE_SIZE];
        getrandom::getrandom(&mut nonce).expect("Failed to generate random nonce");
        nonce
    };

    let mut symmetric_key = derive_symmetric_key(password, salt);

    let mut chacha20 = ChaCha20Poly1305::new(&symmetric_key, &nonce, &aad);

    let (ciphertext, tag) = {
        let mut ciphertext = vec![0u8; data.as_ref().len()];
        let mut tag = [0u8; 16];
        chacha20.encrypt(data.as_ref(), &mut ciphertext, &mut tag);

        (ciphertext.into_boxed_slice(), tag)
    };

    // TODO: something else to zeroize? The password is one option, but I'm not sure if that
    // responsibility should fall here

    symmetric_key.zeroize();
    let builder = TransferSliceBuilder::new();
    builder
        .set_protocol(0x1)?
        .set_salt(&salt)?
        .set_nonce(&nonce)?
        .set_cipher_text(&ciphertext)?
        .set_tag(&tag)
}

pub fn decrypt(password: impl AsRef<[u8]>, data: TransferSlice) -> Result<Box<[u8]>, Error> {
    let aad = [];

    if data.protocol() != 0x1 {
        return Err(Error::InvalidProtocol);
    }

    let mut key = derive_symmetric_key(password, data.salt().try_into().unwrap());

    let mut chacha20 = ChaCha20Poly1305::new(&key, &data.nonce(), &aad);

    let mut plaintext = vec![0u8; data.encrypted_data().len()];
    chacha20.decrypt(data.encrypted_data(), &mut plaintext, data.tag());

    key.zeroize();

    Ok(plaintext.into_boxed_slice())
}

enum SetProtocol {}
enum SetSalt {}
enum SetNonce {}
enum SetCipherText {}
enum SetTag {}

impl TransferSliceBuilder<SetProtocol> {
    fn new() -> Self {
        let codec = Codec::new(vec![]);

        Self(codec, PhantomData)
    }

    fn set_protocol(mut self, protocol: u8) -> Result<TransferSliceBuilder<SetSalt>, Error> {
        self.0.put_u8(protocol)?;
        Ok(TransferSliceBuilder(self.0, PhantomData))
    }
}

impl TransferSliceBuilder<SetSalt> {
    fn set_salt(mut self, salt: &[u8; SALT_SIZE]) -> Result<TransferSliceBuilder<SetNonce>, Error> {
        self.0.put_bytes(salt)?;

        Ok(TransferSliceBuilder(self.0, PhantomData))
    }
}

impl TransferSliceBuilder<SetNonce> {
    fn set_nonce(
        mut self,
        nonce: &[u8; NONCE_SIZE],
    ) -> Result<TransferSliceBuilder<SetCipherText>, Error> {
        self.0.put_bytes(nonce)?;

        Ok(TransferSliceBuilder(self.0, PhantomData))
    }
}

impl TransferSliceBuilder<SetCipherText> {
    fn set_cipher_text(
        mut self,
        cipher_text: impl AsRef<[u8]>,
    ) -> Result<TransferSliceBuilder<SetTag>, Error> {
        if cipher_text.as_ref().len() % 64 != 0 {
            return Err(Error::InvalidDataLength);
        }

        self.0.put_bytes(cipher_text.as_ref())?;
        Ok(TransferSliceBuilder(self.0, PhantomData))
    }
}

impl TransferSliceBuilder<SetTag> {
    fn set_tag(mut self, tag: &[u8; TAG_SIZE]) -> Result<TransferSlice, Error> {
        self.0.put_bytes(tag)?;

        Ok(TransferSlice(self.0.into_inner().into_boxed_slice()))
    }
}

impl TransferSlice {
    #[inline]
    fn protocol(&self) -> u8 {
        self.0[0]
    }

    #[inline]
    fn salt(&self) -> &[u8] {
        &self.0[PROTOCOL_SIZE..PROTOCOL_SIZE + SALT_SIZE]
    }

    #[inline]
    fn nonce(&self) -> &[u8] {
        &self.0[PROTOCOL_SIZE + SALT_SIZE..PROTOCOL_SIZE + SALT_SIZE + NONCE_SIZE]
    }

    fn encrypted_data(&self) -> &[u8] {
        let data_len = self
            .0
            .len()
            .checked_sub(PROTOCOL_SIZE + SALT_SIZE + NONCE_SIZE + TAG_SIZE)
            .unwrap();

        let starting_pos = PROTOCOL_SIZE + SALT_SIZE + NONCE_SIZE;
        &self.0[starting_pos..starting_pos + data_len]
    }

    fn tag(&self) -> &[u8] {
        let start = self.0.len().checked_sub(TAG_SIZE).unwrap();
        &self.0[start..]
    }
}

fn derive_symmetric_key(password: impl AsRef<[u8]>, salt: [u8; SALT_SIZE]) -> [u8; 32] {
    let mut symmetric_key = [0u8; 32];

    let mut mac = Hmac::new(Sha512::new(), password.as_ref());
    pbkdf2(&mut mac, &salt[..], ITERS, &mut symmetric_key);

    symmetric_key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt() {
        let mut bytes: Vec<u8> = vec![];
        bytes.extend([1u8; 64].iter());
        bytes.extend([2u8; 64].iter());
        bytes.extend([3u8; 64].iter());

        let password = [1u8, 2, 3, 4];

        let slice = encrypt(&password, &bytes[..]).unwrap();

        assert_eq!(&decrypt(&password, slice).unwrap()[..], &bytes[..]);
    }

    #[test]
    fn encrypt_non_64_bytes() {
        let bytes = [0u8; 65];
        let password = [1u8, 2, 3, 4];

        assert!(encrypt(&password, &bytes[..]).is_err())
    }

    #[test]
    fn encrypt_decrypt_empty_payload() {
        let bytes = [];
        let password = [1u8, 2, 3, 4];

        assert!(encrypt(&password, &bytes[..]).is_err())
    }
}
