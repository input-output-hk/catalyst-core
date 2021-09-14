use crate::{
    scheme::{on_tx_input, on_tx_output},
    states::States,
    store::UtxoStore,
};
use chain_crypto::{Ed25519, PublicKey};
use chain_impl_mockchain::{
    fragment::{Fragment, FragmentId},
    legacy::OldAddress,
    transaction::{InputEnum, UtxoPointer},
    value::Value,
};
use chain_path_derivation::{
    bip44::{self, Bip44},
    DerivationPath, HardDerivation, SoftDerivation, SoftDerivationRange,
};
use ed25519_bip32::{XPrv, XPub};
use hdkeygen::Key;
use std::{borrow::Borrow, collections::HashMap, hash::Hash};

const DEFAULT_GAG_LIMIT: u32 = 20;

pub struct Wallet<A: 'static> {
    coin_type_key: Key<XPrv, Bip44<bip44::CoinType>>,
    state: States<FragmentId, UtxoStore<Key<XPrv, Bip44<bip44::Address>>>>,
    soft_derivation_range_length: u32,
    mk_key: &'static dyn Fn(&XPub) -> A,
    accounts: Vec<Account<A>>,
}

pub struct Account<A> {
    account: Key<XPub, Bip44<bip44::Account>>,
    next_index: SoftDerivation,
    soft_derivation_range_length: u32,
    addresses: HashMap<A, DerivationPath<Bip44<bip44::Address>>>,
}

impl<A> Account<A> {
    pub fn id(&self) -> HardDerivation {
        self.account.id()
    }
}

impl<A> Account<A>
where
    A: Eq + Hash,
{
    #[inline]
    fn lookup<BA>(&self, address: &BA) -> Option<&DerivationPath<Bip44<bip44::Address>>>
    where
        BA: Eq + Hash,
        A: Borrow<BA>,
    {
        self.addresses.get(address)
    }

    #[inline]
    fn within_last_range(&self, path: &DerivationPath<Bip44<bip44::Address>>) -> bool {
        let idx = path.address();
        let checked = idx.saturating_add(self.soft_derivation_range_length);

        checked >= self.next_index
    }

    fn new_with<F>(account: Key<XPub, Bip44<bip44::Account>>, range: u32, f: F) -> Self
    where
        F: Fn(&XPub) -> A,
    {
        let mut ra = Self {
            account,
            next_index: SoftDerivation::min_value(),
            soft_derivation_range_length: range,
            addresses: HashMap::new(),
        };

        ra.extend_range_with(f);

        ra
    }

    fn extend_range_with<F>(&mut self, f: F)
    where
        F: Fn(&XPub) -> A,
    {
        let start = self.next_index;
        let end = start.saturating_add(self.soft_derivation_range_length);
        self.next_index = end;

        let range = SoftDerivationRange::new(start..end);

        let internal = self.account.internal();
        let external = self.account.external();
        let internal_addresses = internal.addresses(range.clone());
        let external_addresses = external.addresses(range);

        let addresses = internal_addresses.chain(external_addresses);

        for address in addresses {
            let a = f(address.public_key());
            self.addresses.insert(a, address.path().clone());
        }
    }
}

impl<A> Wallet<A> {
    /// confirm a pending transaction
    ///
    /// to only do once it is confirmed a transaction is on chain
    /// and is far enough in the blockchain history to be confirmed
    /// as immutable
    ///
    pub fn confirm(&mut self, fragment_id: &FragmentId) {
        self.state.confirm(fragment_id)
    }

    /// get the confirmed value of the wallet
    pub fn confirmed_value(&self) -> Value {
        self.state.confirmed_state().state().total_value()
    }

    /// get the unconfirmed value of the wallet
    ///
    /// if `None`, it means there is no unconfirmed state of the wallet
    /// and the value can be known from `confirmed_value`.
    ///
    /// The returned value is the value we expect to see at some point on
    /// chain once all transactions are on chain confirmed.
    pub fn unconfirmed_value(&self) -> Option<Value> {
        let s = self.state.last_state();

        Some(s)
            .filter(|s| !s.is_confirmed())
            .map(|s| s.state().total_value())
    }

    /// get all the pending transactions of the wallet
    ///
    /// If empty it means there's no pending transactions waiting confirmation
    ///
    pub fn pending_transactions(&self) -> impl Iterator<Item = &FragmentId> {
        self.state.unconfirmed_states().map(|(k, _)| k)
    }

    /// get the utxos of this given wallet
    pub fn utxos(&self) -> &UtxoStore<Key<XPrv, Bip44<bip44::Address>>> {
        self.state.last_state().state()
    }
}

impl<A> Wallet<A>
where
    A: Eq + Hash,
{
    fn populate_first_account(&mut self) {
        let account_id = HardDerivation::min_value();
        let account = self.coin_type_key.account(account_id);

        let account = Account::<A>::new_with(
            account.public(),
            self.soft_derivation_range_length,
            self.mk_key,
        );
        self.accounts.push(account);
    }

    fn populate_new_account(&mut self) {
        let last_id = self
            .accounts
            .last()
            .expect("there is always one at least")
            .id();

        if let Some(id) = last_id.checked_add(1) {
            let account = self.coin_type_key.account(id);
            let account = Account::<A>::new_with(
                account.public(),
                self.soft_derivation_range_length,
                self.mk_key,
            );
            self.accounts.push(account);
        } else {
            // DO NOTHING... we have reached 2^31 accounts already
        }
    }

    pub(crate) fn check_address(
        &mut self,
        address: &A,
    ) -> Option<DerivationPath<Bip44<bip44::Address>>> {
        let mut accounts = self.accounts.iter_mut();
        let mut result = None;

        for account in &mut accounts {
            if let Some(path) = account.lookup(address).cloned() {
                if account.within_last_range(&path) {
                    account.extend_range_with(self.mk_key);
                }

                result = Some(path);
                break;
            }
        }

        // this is true if we found an address in the last account
        //
        // so we always have 1 account with UTxO ahead
        if result.is_some() && accounts.next().is_none() {
            self.populate_new_account();
        }

        result
    }

    fn check(&mut self, address: &A) -> Option<Key<XPrv, Bip44<bip44::Address>>> {
        let path = self.check_address(address)?;

        let key = self.coin_type_key.account(path.account());
        let key = key.change(path.change());
        let key = key.address(path.address());

        Some(key)
    }
}

impl Wallet<PublicKey<Ed25519>> {
    pub fn from_root_key(coin_type_key: Key<XPrv, Bip44<bip44::CoinType>>) -> Self {
        let mut wallet = Self {
            coin_type_key,
            state: States::new(FragmentId::zero_hash(), UtxoStore::new()),
            soft_derivation_range_length: DEFAULT_GAG_LIMIT,
            mk_key: &mk_public_key,
            accounts: Vec::with_capacity(2),
        };

        wallet.populate_first_account();

        wallet
    }

    pub fn check_fragments<'a, I>(&mut self, fragments: I) -> bool
    where
        I: Iterator<Item = &'a Fragment>,
    {
        let mut at_least_once = false;

        for fragment in fragments {
            let fragment_id = fragment.hash();
            at_least_once |= self.check_fragment(&fragment_id, fragment);
        }

        at_least_once
    }

    pub fn check_fragment(&mut self, fragment_id: &FragmentId, fragment: &Fragment) -> bool {
        if self.state.contains(fragment_id) {
            return true;
        }

        let mut at_least_one_match = false;
        let legacy = self.state.last_state();
        let mut store = legacy.state().clone();

        match fragment {
            Fragment::Initial(_config_params) => {}
            Fragment::UpdateProposal(_update_proposal) => {}
            Fragment::UpdateVote(_signed_update) => {}
            Fragment::OldUtxoDeclaration(_utxos) => {}
            _ => {
                on_tx_input(fragment, |input| {
                    if let InputEnum::UtxoInput(pointer) = input.to_enum() {
                        if let Some(spent) = store.remove(&pointer) {
                            at_least_one_match = true;
                            store = spent;
                        }
                    }
                });

                on_tx_output(fragment, |(index, output)| {
                    use chain_addr::Kind::{Group, Single};
                    let pk = match output.address.kind() {
                        Single(pk) => Some(pk),
                        Group(pk, _) => {
                            // TODO: the account used for the group case
                            // needs to be checked and handled
                            Some(pk)
                        }
                        _ => None,
                    };
                    if let Some(pk) = pk {
                        if let Some(key) = self.check(pk) {
                            let pointer = UtxoPointer {
                                transaction_id: *fragment_id,
                                output_index: index as u8,
                                value: output.value,
                            };

                            store = store.add(pointer, key);
                            at_least_one_match = true;
                        }
                    }
                });
            }
        }

        self.state.push(*fragment_id, store);
        at_least_one_match
    }
}

impl Wallet<OldAddress> {
    pub fn from_root_key(coin_type_key: Key<XPrv, Bip44<bip44::CoinType>>) -> Self {
        let mut wallet = Self {
            coin_type_key,
            state: States::new(FragmentId::zero_hash(), UtxoStore::new()),
            soft_derivation_range_length: DEFAULT_GAG_LIMIT,
            mk_key: &mk_legacy_address,
            accounts: Vec::with_capacity(2),
        };

        wallet.populate_first_account();

        wallet
    }

    pub fn check_fragments<'a, I>(&mut self, fragments: I) -> bool
    where
        I: Iterator<Item = &'a Fragment>,
    {
        let mut at_least_once = false;

        for fragment in fragments {
            let fragment_id = fragment.hash();
            at_least_once |= self.check_fragment(&fragment_id, fragment);
        }

        at_least_once
    }

    pub fn check_fragment(&mut self, fragment_id: &FragmentId, fragment: &Fragment) -> bool {
        if self.state.contains(fragment_id) {
            return true;
        }

        let mut at_least_one_match = false;
        let legacy = self.state.last_state();
        let mut store = legacy.state().clone();

        match fragment {
            Fragment::Initial(_config_params) => {}
            Fragment::UpdateProposal(_update_proposal) => {}
            Fragment::UpdateVote(_signed_update) => {}
            Fragment::OldUtxoDeclaration(utxos) => {
                for (output_index, (address, value)) in utxos.addrs.iter().enumerate() {
                    let pointer = UtxoPointer {
                        transaction_id: *fragment_id,
                        output_index: output_index as u8,
                        value: *value,
                    };

                    if let Some(key) = self.check(address) {
                        at_least_one_match = true;
                        store = store.add(pointer, key);
                    }
                }
            }
            _ => {
                on_tx_input(fragment, |input| {
                    if let InputEnum::UtxoInput(pointer) = input.to_enum() {
                        if let Some(spent) = store.remove(&pointer) {
                            at_least_one_match = true;
                            store = spent;
                        }
                    }
                });
            }
        }

        self.state.push(*fragment_id, store);
        at_least_one_match
    }
}

fn mk_legacy_address(xpub: &XPub) -> OldAddress {
    cardano_legacy_address::ExtendedAddr::new_simple(xpub, None).to_address()
}

fn mk_public_key(xpub: &XPub) -> PublicKey<Ed25519> {
    if let Ok(pk) = PublicKey::from_binary(xpub.public_key_slice()) {
        pk
    } else {
        unsafe { std::hint::unreachable_unchecked() }
    }
}
