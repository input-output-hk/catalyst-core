//! account based wallet, does not really have any Hierarchical element in it
//! thought it is part of the `keygen` entity so adding it in
//!
//! On the chain, an account can group stake while a rindex or a bip44 cannot
//! as they represent individual coins, which have stake but they are not grouped
//! and cannot be be controlled without having an account to group them

use chain_addr::{Address, Discrimination, Kind};
use chain_crypto::{Ed25519, PublicKey};
use cryptoxide::ed25519::{self, PUBLIC_KEY_LENGTH};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

pub use cryptoxide::ed25519::SEED_LENGTH;
pub type SEED = [u8; SEED_LENGTH];

#[derive(Clone)]
pub struct Account {
    seed: SEED,
    counter: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId {
    id: [u8; PUBLIC_KEY_LENGTH],
}

impl Account {
    pub fn account_id(&self) -> AccountId {
        AccountId { id: self.public() }
    }

    pub fn from_seed(seed: SEED) -> Self {
        Account { seed, counter: 0 }
    }

    pub fn public(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        let (_, pk) = ed25519::keypair(&self.seed);
        pk
    }

    /// get the transaction counter
    ///
    /// this is the counter for the number of times a transaction has been successfully sent
    /// to the network with this account. It is used to sign transactions so it is important
    /// to keep it up to date as much as possible.
    pub fn counter(&self) -> u32 {
        self.counter
    }

    pub fn set_counter(&mut self, counter: u32) {
        self.counter = counter;
    }

    /// increase the counter with the given amount
    pub fn increase_counter(&mut self, atm: u32) {
        self.counter += atm
    }

    pub fn seed(&self) -> &SEED {
        &self.seed
    }
}

impl AccountId {
    /// get the public address associated to this account identifier
    pub fn address(&self, discrimination: Discrimination) -> Address {
        let pk = if let Ok(pk) = PublicKey::from_binary(&self.id) {
            pk
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        };
        let kind = Kind::Account(pk);

        Address(discrimination, kind)
    }
}

impl Drop for Account {
    fn drop(&mut self) {
        cryptoxide::util::secure_memset(&mut self.seed, 0)
    }
}

/* Conversion ************************************************************** */

impl From<[u8; SEED_LENGTH]> for Account {
    fn from(seed: [u8; SEED_LENGTH]) -> Self {
        Self { seed, counter: 0 }
    }
}

impl From<[u8; PUBLIC_KEY_LENGTH]> for AccountId {
    fn from(id: [u8; PUBLIC_KEY_LENGTH]) -> Self {
        Self { id }
    }
}

impl Into<PublicKey<Ed25519>> for AccountId {
    fn into(self) -> PublicKey<Ed25519> {
        if let Ok(pk) = PublicKey::from_binary(&self.id) {
            pk
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }
}

/* Display ***************************************************************** */

impl Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        hex::encode(&self.id).fmt(f)
    }
}

impl FromStr for AccountId {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut id = [0; PUBLIC_KEY_LENGTH];

        hex::decode_to_slice(s, &mut id)?;

        Ok(Self { id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Account {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut seed = [0; SEED_LENGTH];
            g.fill_bytes(&mut seed);
            Self { seed, counter: 0 }
        }
    }

    impl Arbitrary for AccountId {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut id = [0; PUBLIC_KEY_LENGTH];
            g.fill_bytes(&mut id);
            Self { id }
        }
    }

    #[quickcheck]
    fn account_id_to_string_parse(account_id: AccountId) -> bool {
        let s = account_id.to_string();
        let decoded = s.parse().unwrap();

        account_id == decoded
    }
}
