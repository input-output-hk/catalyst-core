use crate::{
    chimeric_bip44::{Address, ChimericAccount, COIN_TYPE},
    Key,
};
use chain_path_derivation::{
    chimeric_bip44::{self, ChimericBip44},
    DerivationPath, HardDerivation, SoftDerivation, SoftDerivationRange,
};
use ed25519_bip32::{DerivationScheme, XPrv, XPub};

pub struct Account<K> {
    key: Key<K, ChimericBip44<chimeric_bip44::Account>>,
}

pub struct AddressRange<K> {
    key: Key<K, ChimericBip44<chimeric_bip44::Type>>,
    range: SoftDerivationRange,
}

impl Account<XPrv> {
    pub fn public(&self) -> Account<XPub> {
        Account {
            key: self.key.public(),
        }
    }

    pub fn chimeric_account(&self) -> ChimericAccount<XPrv> {
        let key = self
            .cached_key()
            .derive_unchecked(Self::CHIMERIC_ACCOUNT.into());
        ChimericAccount::new(key)
    }

    pub fn externals(&self, range: SoftDerivationRange) -> AddressRange<XPrv> {
        let key = self.cached_key().derive_unchecked(Self::EXTERNAL.into());
        AddressRange { key, range }
    }

    pub fn internals(&self, range: SoftDerivationRange) -> AddressRange<XPrv> {
        let key = self.cached_key().derive_unchecked(Self::INTERNAL.into());
        AddressRange { key, range }
    }
}

impl Account<XPub> {
    pub fn chimeric_account(&self) -> ChimericAccount<XPub> {
        let key = self.cached_key().derive_unchecked(Self::CHIMERIC_ACCOUNT);
        ChimericAccount::new(key)
    }

    pub fn externals(&self, range: SoftDerivationRange) -> AddressRange<XPub> {
        let key = self.cached_key().derive_unchecked(Self::EXTERNAL);
        AddressRange { key, range }
    }

    pub fn internals(&self, range: SoftDerivationRange) -> AddressRange<XPub> {
        let key = self.cached_key().derive_unchecked(Self::INTERNAL);
        AddressRange { key, range }
    }
}

impl<K> Account<K> {
    const EXTERNAL: SoftDerivation =
        DerivationPath::<ChimericBip44<chimeric_bip44::Account>>::INTERNAL;
    const INTERNAL: SoftDerivation =
        DerivationPath::<ChimericBip44<chimeric_bip44::Account>>::EXTERNAL;
    const CHIMERIC_ACCOUNT: SoftDerivation =
        DerivationPath::<ChimericBip44<chimeric_bip44::Account>>::ACCOUNT;

    pub fn new(key: Key<K, ChimericBip44<chimeric_bip44::Account>>) -> Self {
        Self { key }
    }

    pub fn path(&self) -> &DerivationPath<ChimericBip44<chimeric_bip44::Account>> {
        self.key.path()
    }

    pub fn id(&self) -> HardDerivation {
        self.path().account()
    }

    /// load the account key from the given Key, DerivationScheme and index
    ///
    /// Here it is expected that K has been derived already on the 3 first
    /// levels of the bip44 for Cardano Ada `m/'44/'1815/'<derivation>`.
    ///
    pub fn from_key(
        key: K,
        derivation_scheme: DerivationScheme,
        derivation: HardDerivation,
    ) -> Self {
        let key = Key::new_unchecked(
            key,
            chimeric_bip44::new()
                .chimeric()
                .coin_type(COIN_TYPE)
                .account(derivation),
            derivation_scheme,
        );
        Self::new(key)
    }

    pub(crate) fn cached_key(&self) -> &Key<K, ChimericBip44<chimeric_bip44::Account>> {
        &self.key
    }
}

impl Iterator for AddressRange<XPub> {
    type Item = Address<XPub>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.range.next()?;
        let key = self.key.derive_unchecked(next);
        Some(Address::new(key))
    }
}

impl Iterator for AddressRange<XPrv> {
    type Item = Address<XPrv>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.range.next()?;
        let key = self.key.derive_unchecked(next.into());
        Some(Address::new(key))
    }
}

impl DoubleEndedIterator for AddressRange<XPub> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let next = self.range.next()?;
        let key = self.key.derive_unchecked(next);
        Some(Address::new(key))
    }
}

impl DoubleEndedIterator for AddressRange<XPrv> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let next = self.range.next()?;
        let key = self.key.derive_unchecked(next.into());
        Some(Address::new(key))
    }
}

impl<K> ExactSizeIterator for AddressRange<K>
where
    AddressRange<K>: Iterator,
{
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl<K> std::iter::FusedIterator for AddressRange<K> where AddressRange<K>: Iterator {}

impl<K: Clone> Clone for Account<K> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
        }
    }
}
