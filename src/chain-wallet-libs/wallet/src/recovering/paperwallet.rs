use cryptoxide::hmac::Hmac;
use cryptoxide::pbkdf2::pbkdf2;
use cryptoxide::sha2::Sha512;

const ITERS: u32 = 10000;
pub const IV_SIZE: usize = 8;
const SALT_SIZE: usize = IV_SIZE;

fn gen(iv: &[u8], password: &[u8], buf: &mut [u8]) {
    assert!(iv.len() == IV_SIZE);
    let mut salt = [0u8; SALT_SIZE];
    salt[0..IV_SIZE].copy_from_slice(iv);
    let mut mac = Hmac::new(Sha512::new(), password);
    pbkdf2(&mut mac, &salt[..], ITERS, buf);
}

/// Given a 4 bytes IV, and a password, scramble the input
/// using a simple XOR, and returning the IV prepended to the shielded input
#[cfg(test)]
fn scramble(iv: &[u8], password: &[u8], input: &[u8]) -> Vec<u8> {
    assert!(iv.len() == IV_SIZE);
    let sz = IV_SIZE + input.len();
    let mut out = vec![0; sz];

    out[..IV_SIZE].copy_from_slice(iv);
    gen(iv, password, &mut out[IV_SIZE..sz]);

    for i in IV_SIZE..sz {
        out[i] ^= input[i - IV_SIZE];
    }
    out
}

/// Try to reverse the scramble operation, using
/// the first `IV_SIZE` bytes as IV, and the rest as the shielded input.
pub fn unscramble(password: &[u8], input: &[u8]) -> Vec<u8> {
    assert!(input.len() > IV_SIZE);

    let out_sz = input.len() - IV_SIZE;

    let mut out = vec![0; out_sz];

    gen(&input[0..IV_SIZE], password, &mut out[0..out_sz]);
    for i in 0..out_sz {
        out[i] ^= input[IV_SIZE + i];
    }
    out
}

const PAPERWALLET_CERTIFICATE_MNEMONIC_LENGTH: usize = 27;
const SCRAMBLED_INPUT_WORD_COUNT: usize = 18;

struct InputPassphrase<'a> {
    input: &'a str,
    passphrase: &'a str,
}

// helper function for get_scrambled_input
fn split_scrambled_input_passphrase(mnemonics: &str) -> Option<InputPassphrase> {
    use bip39::dictionary::Language;
    let dic = bip39::dictionary::ENGLISH;

    if mnemonics.split(dic.separator()).count() == PAPERWALLET_CERTIFICATE_MNEMONIC_LENGTH {
        let scrambled_input_end_position = mnemonics
            .match_indices(dic.separator())
            .nth(SCRAMBLED_INPUT_WORD_COUNT - 1)
            .map(|(idx, _sep)| idx)
            .unwrap();

        Some(InputPassphrase {
            input: &mnemonics[..scrambled_input_end_position],
            passphrase: &mnemonics[scrambled_input_end_position + 1..],
        })
    } else {
        None
    }
}

pub fn get_scrambled_input(
    mnemonics: &str,
) -> Result<Option<(&str, bip39::Entropy)>, bip39::Error> {
    let InputPassphrase { input, passphrase } = match split_scrambled_input_passphrase(mnemonics) {
        Some(input_passphrase) => input_passphrase,
        None => return Ok(None),
    };
    let dic = bip39::dictionary::ENGLISH;
    let mnemonics = bip39::Mnemonics::from_string(&dic, input)?;
    let entropy = bip39::Entropy::from_mnemonics(&mnemonics)?;

    Ok(Some((passphrase, entropy)))
}

#[cfg(test)]
mod tests {
    //use paperwallet::{scramble,unscramble};
    use crate::recovering::paperwallet;

    /// # GOLDEN_TEST: cardano/crypto/scramble128
    ///
    ///
    ///
    /// ## Input(s)
    ///
    /// ```text
    /// iv ([u8,4]) = "hexadecimal encoded bytes"
    /// input (&'static str) = "UTF8 BIP39 passphrase (english)"
    /// passphrase (&'static str) = "Bouble quoted, encoded string."
    /// ```
    ///
    /// ## Output(s)
    ///
    /// ```text
    /// shielded_input (&'static str) = "UTF8 BIP39 passphrase (english)"
    /// ```
    struct TestVector {
        iv: [u8; 8],
        input: [u8; 16],
        passphrase: &'static str,
        shielded_input: [u8; 24],
    }

    const GOLDEN_TESTS : [TestVector;3] =
  [ TestVector
    { iv : [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    , input : [0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f]
    , passphrase : ""
    , shielded_input : [0, 0, 0, 0, 0, 0, 0, 0, 250, 194, 41, 40, 102, 196, 34, 60, 90, 125, 175, 186, 222, 152, 14, 9]
    }
  , TestVector
    { iv : [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]
    , input : [0x5a, 0x94, 0x0d, 0x50, 0xab, 0x0d, 0x4e, 0x2e, 0xbf, 0x3b, 0x2c, 0x6e, 0xb3, 0x99, 0xe8, 0x27]
    , passphrase : "Cardano Ada"
    , shielded_input : [0, 1, 2, 3, 4, 5, 6, 7, 193, 34, 111, 15, 127, 245, 15, 164, 3, 24, 171, 35, 99, 32, 181, 158]
    }
  , TestVector
    { iv : [0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a]
    , input : [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
    , passphrase : "This is a very long passphrase. This is a very long passphrase. This is a very long passphrase. This is a very long passphrase."
    , shielded_input : [42, 42, 42, 42, 42, 42, 42, 42, 199, 113, 24, 116, 236, 196, 179, 147, 0, 136, 72, 43, 59, 108, 139, 133]
    }
  ];

    #[test]
    fn paper_scramble() {
        for tv in GOLDEN_TESTS.iter() {
            let r = paperwallet::scramble(&tv.iv[..], tv.passphrase.as_bytes(), &tv.input[..]);
            assert_eq!(
                &r[..],
                &tv.shielded_input[..],
                "scramble(iv: {iv}, password: {password}, input: {input}): {output} != {expected}",
                iv = hex::encode(&tv.iv),
                password = tv.passphrase,
                input = hex::encode(&tv.input),
                output = hex::encode(&r),
                expected = hex::encode(&tv.shielded_input),
            );
        }
    }

    #[test]
    fn paper_unscramble() {
        for tv in GOLDEN_TESTS.iter() {
            let r = paperwallet::unscramble(tv.passphrase.as_bytes(), &tv.shielded_input[..]);
            assert_eq!(&r[..], &tv.input[..]);
        }
    }
}
