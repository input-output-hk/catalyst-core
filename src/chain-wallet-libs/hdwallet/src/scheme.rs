//! wallet scheme interfaces. provide common interfaces to manage wallets
//! generate addresses and sign transactions.
//!

/// main wallet scheme, provides all the details to manage a wallet:
/// from managing wallet [`Account`](./trait.Account.html)s and
/// signing transactions.
///
pub trait Wallet {
    /// associated `Account` type, must implement the [`Account`](./trait.Account.html)
    /// trait.
    type Account: Account;

    /// the associated type for the stored accounts. Some wallet may
    /// provide different model to handle accounts.
    ///
    type Accounts;

    /// addressing model associated to this wallet scheme.
    ///
    /// provides a description about how to derive a public key
    /// from a wallet point of view.
    type Addressing: Clone;

    /// create an account with the associated alias.
    ///
    /// The alias may not be used in some wallets which does not support
    /// accounts such as the daedalus wallet.
    ///
    fn create_account(&mut self, alias: &str, id: u32) -> Self::Account;

    /// list all the accounts known of this wallet
    fn list_accounts<'a>(&'a self) -> &'a Self::Accounts;
}

/// account level scheme, provides all the details to manage an account:
/// i.e. generate new addresses associated to this account.
pub trait Account {
    /// addressing model associated to this account scheme.
    ///
    /// provides a description about how to derive a public key
    /// from a wallet point of view.
    type Addressing;
    type PublicKey;

    fn generate_key<'a, I>(&'a self, addresses: I) -> Vec<Self::PublicKey>
    where
        I: Iterator<Item = &'a Self::Addressing>;
}
