use crate::{Key, KeyRange};
use chain_addr::{Discrimination, Kind};
use chain_path_derivation::{
    bip44::{self, Bip44},
    Derivation, DerivationPath, HardDerivation, SoftDerivation, SoftDerivationRange,
};
use ed25519_bip32::{XPrv, XPub};

impl Key<XPrv, Bip44<bip44::Root>> {
    pub fn purpose(&self, derivation: HardDerivation) -> Key<XPrv, Bip44<bip44::Purpose>> {
        self.derive_unchecked(derivation.into())
    }

    pub fn bip44(&self) -> Key<XPrv, Bip44<bip44::Purpose>> {
        self.purpose(bip44::PURPOSE_BIP44)
    }

    pub fn chimeric_bip44(&self) -> Key<XPrv, Bip44<bip44::Purpose>> {
        self.purpose(bip44::PURPOSE_CHIMERIC)
    }
}

impl Key<XPrv, Bip44<bip44::Purpose>> {
    pub fn coin_type(&self, derivation: HardDerivation) -> Key<XPrv, Bip44<bip44::CoinType>> {
        self.derive_unchecked(derivation.into())
    }

    pub fn cardano(&self) -> Key<XPrv, Bip44<bip44::CoinType>> {
        const COIN_TYPE: HardDerivation =
            HardDerivation::new_unchecked(Derivation::new(0x8000_0717));
        self.coin_type(COIN_TYPE)
    }
}

impl Key<XPrv, Bip44<bip44::CoinType>> {
    pub fn account(&self, derivation: HardDerivation) -> Key<XPrv, Bip44<bip44::Account>> {
        self.derive_unchecked(derivation.into())
    }
}

impl<K> Key<K, Bip44<bip44::Account>> {
    const EXTERNAL: SoftDerivation = DerivationPath::<Bip44<bip44::Account>>::EXTERNAL;
    const INTERNAL: SoftDerivation = DerivationPath::<Bip44<bip44::Account>>::INTERNAL;
    const ACCOUNT: SoftDerivation = DerivationPath::<Bip44<bip44::Account>>::ACCOUNT;

    pub fn id(&self) -> HardDerivation {
        self.path().account()
    }
}

impl Key<XPrv, Bip44<bip44::Account>> {
    pub fn change(&self, derivation: SoftDerivation) -> Key<XPrv, Bip44<bip44::Change>> {
        self.derive_unchecked(derivation.into())
    }

    pub fn external(&self) -> Key<XPrv, Bip44<bip44::Change>> {
        self.change(Self::EXTERNAL)
    }

    pub fn internal(&self) -> Key<XPrv, Bip44<bip44::Change>> {
        self.change(Self::INTERNAL)
    }

    pub fn account(&self) -> Key<XPrv, Bip44<bip44::Change>> {
        self.change(Self::ACCOUNT)
    }
}

impl Key<XPub, Bip44<bip44::Account>> {
    pub fn change(&self, derivation: SoftDerivation) -> Key<XPub, Bip44<bip44::Change>> {
        self.derive_unchecked(derivation)
    }

    pub fn external(&self) -> Key<XPub, Bip44<bip44::Change>> {
        self.change(Self::EXTERNAL)
    }

    pub fn internal(&self) -> Key<XPub, Bip44<bip44::Change>> {
        self.change(Self::INTERNAL)
    }

    pub fn account(&self) -> Key<XPub, Bip44<bip44::Change>> {
        self.change(Self::ACCOUNT)
    }
}

impl Key<XPrv, Bip44<bip44::Change>> {
    pub fn address(&self, derivation: SoftDerivation) -> Key<XPrv, Bip44<bip44::Address>> {
        self.derive_unchecked(derivation.into())
    }
}

impl Key<XPub, Bip44<bip44::Change>> {
    pub fn address(&self, derivation: SoftDerivation) -> Key<XPub, Bip44<bip44::Address>> {
        self.derive_unchecked(derivation)
    }

    pub fn addresses(
        &self,
        range: SoftDerivationRange,
    ) -> KeyRange<XPub, SoftDerivationRange, Bip44<bip44::Change>, Bip44<bip44::Address>> {
        KeyRange::new(self, range)
    }
}

impl Key<XPub, Bip44<bip44::Address>> {
    pub fn address_single(&self, discrimination: Discrimination) -> chain_addr::Address {
        let pk = self.pk();
        let kind = Kind::Single(pk);

        chain_addr::Address(discrimination, kind)
    }

    pub fn address_account(&self, discrimination: Discrimination) -> chain_addr::Address {
        let pk = self.pk();
        let kind = Kind::Account(pk);

        chain_addr::Address(discrimination, kind)
    }

    pub fn address_group(
        &self,
        discrimination: Discrimination,
        group: chain_crypto::PublicKey<chain_crypto::Ed25519>,
    ) -> chain_addr::Address {
        let pk = self.pk();
        let kind = Kind::Group(pk, group);

        chain_addr::Address(discrimination, kind)
    }
}
