//! BIP44 derivation scheme and address model

use bip39;
use bip44::{BIP44_COIN_TYPE, BIP44_PURPOSE, BIP44_SOFT_UPPER_BOUND};

use ed25519_bip32::{DerivationError, DerivationIndex, DerivationScheme, XPrv, XPub, XPRV_SIZE};
use std::{collections::BTreeMap, ops::Deref};

use super::keygen;
use super::scheme;

pub use bip44::{self, AddrType, Addressing, Change, Error, Index};

/// BIP44 based wallet, i.e. using sequential indexing.
///
/// See [BIP44](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)
/// specifications for more details.
///
pub struct Wallet {
    cached_root_key: RootLevel<XPrv>,
    accounts: BTreeMap<String, Account<XPrv>>,
    derivation_scheme: DerivationScheme,
}
impl Wallet {
    /// load a wallet from a cached root key
    ///
    /// this is handy to reconstruct the wallet from a locally saved
    /// state (beware that the cached root key would need to be stored
    /// in a secure manner though).
    ///
    pub fn from_cached_key(
        cached_root_key: RootLevel<XPrv>,
        derivation_scheme: DerivationScheme,
    ) -> Self {
        let accounts = BTreeMap::new();
        Wallet {
            cached_root_key,
            accounts,
            derivation_scheme,
        }
    }

    /// construct a new `Wallet` from the given Root key. Not really meant
    /// to reconstruct the wallet from locally saved state, but more to allow
    /// generating root seed without using bip39 mnemonics as proposed in
    /// [`Wallet::from_bip39_mnemonics`](./struct.Wallet.html#method.from_bip39_mnemonics)
    /// constructor.
    ///
    pub fn from_root_key(root_key: XPrv, derivation_scheme: DerivationScheme) -> Self {
        let cached_root_key = root_key
            .derive(derivation_scheme, BIP44_PURPOSE)
            .derive(derivation_scheme, BIP44_COIN_TYPE);
        Wallet::from_cached_key(RootLevel::from(cached_root_key), derivation_scheme)
    }

    /// helper to create a wallet from BIP39 Seed
    ///
    /// We assume the [`MnemonicString`](../../bip/bip39/struct.MnemonicString.html)
    /// so we don't have to handle error in this constructor.
    ///
    /// Prefer `from_entropy` unless BIP39 seed generation compatibility is needed.
    pub fn from_bip39_seed(seed: &bip39::Seed, derivation_scheme: DerivationScheme) -> Self {
        let mut sk = [0; 32];
        sk.clone_from_slice(&seed.as_ref()[0..32]);
        let mut cc = [0; 32];
        cc.clone_from_slice(&seed.as_ref()[32..64]);
        let xprv = XPrv::from_nonextended(sk, cc);

        Wallet::from_root_key(xprv, derivation_scheme)
    }

    /*
    pub fn generate_from_bip39(bytes: &bip39::Seed) -> Self {
        let mut out = [0u8; XPRV_SIZE];

        mk_ed25519_extended(&mut out[0..64], &bytes.as_ref()[0..32]);
        out[31] &= 0b1101_1111; // set 3rd highest bit to 0 as per the spec
        out[64..96].clone_from_slice(&bytes.as_ref()[32..64]);

        Self::from_bytes(out)
    }
    */

    /// helper to create a wallet from BIP39 mnemonics
    ///
    /// We assume the [`MnemonicString`](../../bip/bip39/struct.MnemonicString.html)
    /// so we don't have to handle error in this constructor.
    ///
    /// Prefer `from_entropy` unless BIP39 seed generation compatibility is needed.
    pub fn from_bip39_mnemonics(
        mnemonics_phrase: &bip39::MnemonicString,
        password: &[u8],
        derivation_scheme: DerivationScheme,
    ) -> Self {
        let seed = bip39::Seed::from_mnemonic_string(mnemonics_phrase, password);

        Wallet::from_bip39_seed(&seed, derivation_scheme)
    }

    /// Create a new wallet from a root entropy
    ///
    /// This is the recommended method to create a wallet from initial generated value.
    ///
    /// Note this method, doesn't put the bip39 dictionary used in the cryptographic data,
    /// hence the way the mnemonics are displayed is independent of the language chosen.
    pub fn from_entropy(
        entropy: &bip39::Entropy,
        password: &[u8],
        derivation_scheme: DerivationScheme,
    ) -> Self {
        let mut seed = [0u8; XPRV_SIZE];
        keygen::generate_seed(entropy, password, &mut seed);
        let xprv = XPrv::normalize_bytes(seed);
        Wallet::from_root_key(xprv, derivation_scheme)
    }

    pub fn derivation_scheme(&self) -> DerivationScheme {
        self.derivation_scheme
    }
}
impl Deref for Wallet {
    type Target = RootLevel<XPrv>;
    fn deref(&self) -> &Self::Target {
        &self.cached_root_key
    }
}
impl scheme::Wallet for Wallet {
    type Account = Account<XPrv>;
    type Accounts = BTreeMap<String, Self::Account>;
    type Addressing = Addressing;

    fn create_account(&mut self, alias: &str, id: u32) -> Self::Account {
        let account = self.cached_root_key.account(self.derivation_scheme, id);
        let account = Account {
            cached_root_key: account,
            derivation_scheme: self.derivation_scheme,
        };
        self.accounts.insert(alias.to_owned(), account.clone());
        account
    }
    fn list_accounts<'a>(&'a self) -> &'a Self::Accounts {
        &self.accounts
    }
}

#[derive(Clone)]
pub struct Account<K> {
    cached_root_key: AccountLevel<K>,
    derivation_scheme: DerivationScheme,
}
impl<K> Account<K> {
    pub fn new(cached_root_key: AccountLevel<K>, derivation_scheme: DerivationScheme) -> Self {
        Account {
            cached_root_key,
            derivation_scheme,
        }
    }
}
impl Account<XPrv> {
    pub fn public(&self) -> Account<XPub> {
        Account {
            cached_root_key: self.cached_root_key.public(),
            derivation_scheme: self.derivation_scheme,
        }
    }
}
impl Deref for Account<XPrv> {
    type Target = AccountLevel<XPrv>;
    fn deref(&self) -> &Self::Target {
        &self.cached_root_key
    }
}
impl Deref for Account<XPub> {
    type Target = AccountLevel<XPub>;
    fn deref(&self) -> &Self::Target {
        &self.cached_root_key
    }
}
impl scheme::Account for Account<XPub> {
    type Addressing = (bip44::AddrType, u32);
    type PublicKey = IndexLevel<XPub>;

    fn generate_key<'a, I>(&'a self, addresses: I) -> Vec<Self::PublicKey>
    where
        I: Iterator<Item = &'a Self::Addressing>,
    {
        let (hint_low, hint_max) = addresses.size_hint();
        let mut vec = Vec::with_capacity(hint_max.unwrap_or(hint_low));

        for addressing in addresses {
            let key = self
                .cached_root_key
                .change(self.derivation_scheme, addressing.0)
                .expect("cannot fail")
                .index(self.derivation_scheme, addressing.1)
                .expect("cannot fail");
            vec.push(key);
        }
        vec
    }
}
impl scheme::Account for Account<XPrv> {
    type Addressing = (bip44::AddrType, u32);
    type PublicKey = IndexLevel<XPub>;

    fn generate_key<'a, I>(&'a self, addresses: I) -> Vec<Self::PublicKey>
    where
        I: Iterator<Item = &'a Self::Addressing>,
    {
        let (hint_low, hint_max) = addresses.size_hint();
        let mut vec = Vec::with_capacity(hint_max.unwrap_or(hint_low));

        for addressing in addresses {
            let key = self
                .cached_root_key
                .change(self.derivation_scheme, addressing.0)
                .index(self.derivation_scheme, addressing.1)
                .public();
            vec.push(key);
        }
        vec
    }
}

/// create an `AddressGenerator`
///
/// an address iterator starts from the given index, and stop when
/// the last soft derivation is reached
/// ([`BIP44_SOFT_UPPER_BOUND`](../../bip/bip44/constant.BIP44_SOFT_UPPER_BOUND.html)).
///
/// see [`Account<XPrv>::address_generator`](./struct.Account.html#method.address_generator)
/// and [`Account<XPub>::address_generator`](./struct.Account.html#method.address_generator-1)
/// for example of use.
///
pub struct AddressGenerator<K> {
    cached_root_key: ChangeLevel<K>,
    derivation_scheme: DerivationScheme,
    index: u32,
}
impl Iterator for AddressGenerator<XPrv> {
    type Item = IndexLevel<XPrv>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= BIP44_SOFT_UPPER_BOUND {
            return None;
        }
        let index = self.index;
        self.index += 1;

        let index = self.cached_root_key.index(self.derivation_scheme, index);
        Some(index)
    }
}
impl Iterator for AddressGenerator<XPub> {
    type Item = Result<IndexLevel<XPub>, DerivationError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= BIP44_SOFT_UPPER_BOUND {
            return None;
        }
        let index = self.index;
        self.index += 1;

        let index = self.cached_root_key.index(self.derivation_scheme, index);
        Some(index)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootLevel<T>(T);
impl RootLevel<XPrv> {
    pub fn account(&self, derivation_scheme: DerivationScheme, id: u32) -> AccountLevel<XPrv> {
        AccountLevel::from(
            self.0
                .derive(derivation_scheme, BIP44_SOFT_UPPER_BOUND | id),
        )
    }
}
impl<T> Deref for RootLevel<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
impl From<XPrv> for RootLevel<XPrv> {
    fn from(xprv: XPrv) -> Self {
        RootLevel(xprv)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountLevel<T>(T);
impl AccountLevel<XPrv> {
    pub fn external(&self, derivation_scheme: DerivationScheme) -> ChangeLevel<XPrv> {
        ChangeLevel::from(self.0.derive(derivation_scheme, 0))
    }
    pub fn internal(&self, derivation_scheme: DerivationScheme) -> ChangeLevel<XPrv> {
        ChangeLevel::from(self.0.derive(derivation_scheme, 1))
    }
    pub fn change(
        &self,
        derivation_scheme: DerivationScheme,
        addr_type: AddrType,
    ) -> ChangeLevel<XPrv> {
        match addr_type {
            AddrType::Internal => self.internal(derivation_scheme),
            AddrType::External => self.external(derivation_scheme),
        }
    }
    pub fn public(&self) -> AccountLevel<XPub> {
        AccountLevel::from(self.0.public())
    }
}
impl From<XPrv> for AccountLevel<XPrv> {
    fn from(xprv: XPrv) -> Self {
        AccountLevel(xprv)
    }
}
impl AccountLevel<XPub> {
    pub fn internal(
        &self,
        derivation_scheme: DerivationScheme,
    ) -> Result<ChangeLevel<XPub>, DerivationError> {
        Ok(ChangeLevel::from(self.0.derive(derivation_scheme, 1)?))
    }
    pub fn external(
        &self,
        derivation_scheme: DerivationScheme,
    ) -> Result<ChangeLevel<XPub>, DerivationError> {
        Ok(ChangeLevel::from(self.0.derive(derivation_scheme, 0)?))
    }
    pub fn change(
        &self,
        derivation_scheme: DerivationScheme,
        addr_type: AddrType,
    ) -> Result<ChangeLevel<XPub>, DerivationError> {
        match addr_type {
            AddrType::Internal => self.internal(derivation_scheme),
            AddrType::External => self.external(derivation_scheme),
        }
    }
}
impl From<XPub> for AccountLevel<XPub> {
    fn from(xpub: XPub) -> Self {
        AccountLevel(xpub)
    }
}
impl<T> Deref for AccountLevel<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeLevel<T>(T);
impl ChangeLevel<XPrv> {
    pub fn index(
        &self,
        derivation_scheme: DerivationScheme,
        index: DerivationIndex,
    ) -> IndexLevel<XPrv> {
        IndexLevel::from(self.0.derive(derivation_scheme, index))
    }
    pub fn public(&self) -> ChangeLevel<XPub> {
        ChangeLevel::from(self.0.public())
    }
}
impl From<XPrv> for ChangeLevel<XPrv> {
    fn from(xprv: XPrv) -> Self {
        ChangeLevel(xprv)
    }
}
impl ChangeLevel<XPub> {
    pub fn index(
        &self,
        derivation_scheme: DerivationScheme,
        index: DerivationIndex,
    ) -> Result<IndexLevel<XPub>, DerivationError> {
        Ok(IndexLevel::from(self.0.derive(derivation_scheme, index)?))
    }
}
impl From<XPub> for ChangeLevel<XPub> {
    fn from(xpub: XPub) -> Self {
        ChangeLevel(xpub)
    }
}
impl<T> Deref for ChangeLevel<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexLevel<T>(T);
impl IndexLevel<XPrv> {
    pub fn public(&self) -> IndexLevel<XPub> {
        IndexLevel::from(self.0.public())
    }
}
impl From<XPrv> for IndexLevel<XPrv> {
    fn from(xprv: XPrv) -> Self {
        IndexLevel(xprv)
    }
}
impl From<XPub> for IndexLevel<XPub> {
    fn from(xpub: XPub) -> Self {
        IndexLevel(xpub)
    }
}
impl<T> Deref for IndexLevel<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
