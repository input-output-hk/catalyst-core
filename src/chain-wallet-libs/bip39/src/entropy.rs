use crate::{Error, MnemonicIndex, Mnemonics, Result, Type, MAX_MNEMONIC_VALUE};
use std::ops::Deref;

/// BIP39 entropy is used as root entropy for the HDWallet PRG
/// to generate the HDWallet root keys.
///
/// See module documentation for mode details about how to use
/// `Entropy`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, zeroize::ZeroizeOnDrop)]
pub enum Entropy {
    Entropy9([u8; 12]),
    Entropy12([u8; 16]),
    Entropy15([u8; 20]),
    Entropy18([u8; 24]),
    Entropy21([u8; 28]),
    Entropy24([u8; 32]),
}

impl Entropy {
    /// Retrieve an `Entropy` from the given slice.
    ///
    /// # Error
    ///
    /// This function may fail if the given slice's length is not
    /// one of the supported entropy length. See [`Type`](./enum.Type.html)
    /// for the list of supported entropy sizes.
    ///
    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        let t = Type::from_entropy_size(bytes.len() * 8)?;
        Ok(Self::new(t, bytes))
    }

    /// generate entropy using the given random generator.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate rand;
    /// # use bip39::*;
    ///
    /// let entropy = Entropy::generate(Type::Type15Words, rand::random);
    /// ```
    ///
    pub fn generate<G>(t: Type, gen: G) -> Self
    where
        G: Fn() -> u8,
    {
        let bytes = [0u8; 32];
        let mut entropy = Self::new(t, &bytes[..]);
        for e in entropy.as_mut().iter_mut() {
            *e = gen();
        }
        entropy
    }

    fn new(t: Type, bytes: &[u8]) -> Self {
        let mut e = match t {
            Type::Type9Words => Entropy::Entropy9([0u8; 12]),
            Type::Type12Words => Entropy::Entropy12([0u8; 16]),
            Type::Type15Words => Entropy::Entropy15([0u8; 20]),
            Type::Type18Words => Entropy::Entropy18([0u8; 24]),
            Type::Type21Words => Entropy::Entropy21([0u8; 28]),
            Type::Type24Words => Entropy::Entropy24([0u8; 32]),
        };
        let len = e.as_ref().len();
        e.as_mut()[..len].copy_from_slice(&bytes[..len]);
        e
    }

    /// handy helper to retrieve the [`Type`](./enum.Type.html)
    /// from the `Entropy`.
    #[inline]
    pub fn get_type(&self) -> Type {
        match self {
            Entropy::Entropy9(_) => Type::Type9Words,
            Entropy::Entropy12(_) => Type::Type12Words,
            Entropy::Entropy15(_) => Type::Type15Words,
            Entropy::Entropy18(_) => Type::Type18Words,
            Entropy::Entropy21(_) => Type::Type21Words,
            Entropy::Entropy24(_) => Type::Type24Words,
        }
    }

    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Entropy::Entropy9(ref mut b) => b.as_mut(),
            Entropy::Entropy12(ref mut b) => b.as_mut(),
            Entropy::Entropy15(ref mut b) => b.as_mut(),
            Entropy::Entropy18(ref mut b) => b.as_mut(),
            Entropy::Entropy21(ref mut b) => b.as_mut(),
            Entropy::Entropy24(ref mut b) => b.as_mut(),
        }
    }

    fn hash(&self) -> [u8; 32] {
        use cryptoxide::digest::Digest;
        use cryptoxide::sha2::Sha256;
        let mut hasher = Sha256::new();
        let mut res = [0u8; 32];
        hasher.input(self.as_ref());
        hasher.result(&mut res);
        res
    }

    /// compute the checksum of the entropy, be aware that only
    /// part of the bytes may be useful for the checksum depending
    /// of the [`Type`](./enum.Type.html) of the `Entropy`.
    ///
    /// | entropy type | checksum size (in bits) |
    /// | ------------ | ----------------------- |
    /// | 9 words      | 3 bits                  |
    /// | 12 words     | 4 bits                  |
    /// | 15 words     | 5 bits                  |
    /// | 18 words     | 6 bits                  |
    /// | 21 words     | 7 bits                  |
    /// | 24 words     | 8 bits                  |
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate rand;
    /// # use bip39::*;
    ///
    /// let entropy = Entropy::generate(Type::Type15Words, rand::random);
    ///
    /// let checksum = entropy.checksum() & 0b0001_1111;
    /// ```
    ///
    pub fn checksum(&self) -> u8 {
        let hash = self.hash()[0];
        match self.get_type() {
            Type::Type9Words => (hash >> 5) & 0b0000_0111,
            Type::Type12Words => (hash >> 4) & 0b0000_1111,
            Type::Type15Words => (hash >> 3) & 0b0001_1111,
            Type::Type18Words => (hash >> 2) & 0b0011_1111,
            Type::Type21Words => (hash >> 1) & 0b0111_1111,
            Type::Type24Words => hash,
        }
    }

    /// retrieve the `Entropy` from the given [`Mnemonics`](./struct.Mnemonics.html).
    ///
    /// # Example
    ///
    /// ```
    /// # use bip39::*;
    ///
    /// const MNEMONICS : &'static str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    /// let mnemonics = Mnemonics::from_string(&dictionary::ENGLISH, MNEMONICS)
    ///     .expect("validating the given mnemonics phrase");
    ///
    /// let entropy = Entropy::from_mnemonics(&mnemonics)
    ///     .expect("retrieving the entropy from the mnemonics");
    /// ```
    ///
    /// # Error
    ///
    /// This function may fail if the Mnemonic has an invalid checksum. As part of the
    /// BIP39, the checksum must be embedded in the mnemonic phrase. This allow to check
    /// the mnemonics have been correctly entered by the user.
    ///
    pub fn from_mnemonics(mnemonics: &Mnemonics) -> Result<Self> {
        use super::bits::BitWriterBy11;
        let t = mnemonics.get_type();

        let mut to_validate = BitWriterBy11::new();
        for mnemonic in mnemonics.iter() {
            to_validate.write(mnemonic.0);
        }

        let mut r = to_validate.into_vec();

        let entropy_bytes = Vec::from(&r[..t.to_key_size() / 8]);
        let entropy = Self::new(t, &entropy_bytes[..]);
        if let Some(h) = r.pop() {
            let h2 = h >> (8 - t.checksum_size_bits());
            let cs = entropy.checksum();
            if cs != h2 {
                return Err(Error::InvalidChecksum(cs, h2));
            }
        };

        Ok(entropy)
    }

    /// convert the given `Entropy` into a mnemonic phrase.
    ///
    /// # Example
    ///
    /// ```
    /// # use bip39::*;
    ///
    /// let entropy = Entropy::Entropy12([0;16]);
    ///
    /// let mnemonics = entropy.to_mnemonics()
    ///     .to_string(&dictionary::ENGLISH);
    /// ```
    ///
    pub fn to_mnemonics(&self) -> Mnemonics {
        use super::bits::BitReaderBy11;

        let t = self.get_type();
        let mut combined = Vec::from(self.as_ref());
        combined.extend(&self.hash()[..]);

        let mut reader = BitReaderBy11::new(&combined);

        let mut words: Vec<MnemonicIndex> = Vec::new();
        for _ in 0..t.mnemonic_count() {
            // here we are confident the entropy has already
            // enough bytes to read all the bits we need.
            let n = reader.read();
            // assert only in non optimized builds, Since we read 11bits
            // by 11 bits we should not allow values beyond 2047.
            debug_assert!( n <= MAX_MNEMONIC_VALUE
                         , "Something went wrong, the BitReaderBy11 did return an impossible value: {} (0b{:016b})"
                         , n, n
                         );
            // here we can unwrap safely as 11bits can
            // only store up to the value 2047
            words.push(MnemonicIndex::new(n).unwrap());
        }
        // by design, it is safe to call unwrap here as
        // the mnemonic length has been validated by construction.
        Mnemonics::from_mnemonics(words).unwrap()
    }
}

impl AsRef<[u8]> for Entropy {
    fn as_ref(&self) -> &[u8] {
        match self {
            Entropy::Entropy9(ref b) => b.as_ref(),
            Entropy::Entropy12(ref b) => b.as_ref(),
            Entropy::Entropy15(ref b) => b.as_ref(),
            Entropy::Entropy18(ref b) => b.as_ref(),
            Entropy::Entropy21(ref b) => b.as_ref(),
            Entropy::Entropy24(ref b) => b.as_ref(),
        }
    }
}

impl Deref for Entropy {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
