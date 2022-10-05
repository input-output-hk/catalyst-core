//! account based wallet, does not really have any Hierarchical element in it
//! thought it is part of the `keygen` entity so adding it in
//!
//! On the chain, an account can group stake while a rindex or a bip44 cannot
//! as they represent individual coins, which have stake but they are not grouped
//! and cannot be be controlled without having an account to group them

use chain_addr::{Address, Discrimination, Kind};
use chain_crypto::{AsymmetricKey, Ed25519, Ed25519Extended, PublicKey, SecretKey};
use cryptoxide::ed25519::{self, PRIVATE_KEY_LENGTH};
use std::{
    convert::TryInto,
    fmt::{self, Display},
    str::FromStr,
};

pub type Seed = [u8; PRIVATE_KEY_LENGTH];

pub struct Account<K: AsymmetricKey> {
    secret: SecretKey<K>,
    counter: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId {
    id: [u8; AccountId::SIZE],
}

impl Account<Ed25519> {
    pub fn from_seed(seed: Seed) -> Self {
        let secret = SecretKey::<Ed25519>::from_binary(&seed).unwrap();
        Account { secret, counter: 0 }
    }
}

impl Account<Ed25519Extended> {
    pub fn from_secret_key(key: SecretKey<Ed25519Extended>) -> Self {
        Account {
            secret: key,
            counter: 0,
        }
    }
}

impl<K: AsymmetricKey> Account<K> {
    pub fn account_id(&self) -> AccountId {
        AccountId { id: self.public() }
    }

    pub fn public(&self) -> [u8; AccountId::SIZE] {
        self.secret.to_public().as_ref().try_into().unwrap()
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

    pub fn secret(&self) -> &SecretKey<K> {
        &self.secret
    }

    // pub fn seed(&self) -> &SEED {
    //     &self.seed
    // }
}

impl AccountId {
    /// the total size of an account ID
    pub const SIZE: usize = ed25519::PUBLIC_KEY_LENGTH;

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

// impl Drop for Account {
//     fn drop(&mut self) {
//         cryptoxide::util::secure_memset(&mut self.seed, 0)
//     }
// }

/* Conversion ************************************************************** */

// impl From<[u8; SEED_LENGTH]> for Account {
//     fn from(seed: [u8; SEED_LENGTH]) -> Self {
//         Self { seed, counter: 0 }
//     }
// }

impl From<[u8; Self::SIZE]> for AccountId {
    fn from(id: [u8; Self::SIZE]) -> Self {
        Self { id }
    }
}

impl From<AccountId> for PublicKey<Ed25519> {
    fn from(account: AccountId) -> PublicKey<Ed25519> {
        if let Ok(pk) = PublicKey::from_binary(&account.id) {
            pk
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }
}

impl AsRef<[u8]> for AccountId {
    fn as_ref(&self) -> &[u8] {
        self.id.as_ref()
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
        let mut id = [0; Self::SIZE];

        hex::decode_to_slice(s, &mut id)?;

        Ok(Self { id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Clone for Account<Ed25519> {
        fn clone(&self) -> Self {
            Self {
                secret: self.secret.clone(),
                counter: self.counter,
            }
        }
    }

    impl Arbitrary for Account<Ed25519> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut seed = [0; PRIVATE_KEY_LENGTH];
            g.fill_bytes(&mut seed);
            Self::from_seed(seed)
        }
    }

    impl Arbitrary for AccountId {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut id = [0; Self::SIZE];
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
