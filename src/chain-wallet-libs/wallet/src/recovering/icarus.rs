use crate::recovering::{dump::WitnessBuilder, Dump};
use chain_impl_mockchain::{
    fragment::Fragment,
    legacy::OldAddress,
    transaction::{Input, UtxoPointer},
    value::Value,
};
use chain_path_derivation::{
    bip44::{self, Bip44},
    DerivationPath, HardDerivation, SoftDerivation, SoftDerivationRange,
};
use ed25519_bip32::XPrv;
use hdkeygen::bip44::{Account, Address, Wallet};
use std::collections::HashMap;

const CHANGE_EXTERNAL: SoftDerivation = DerivationPath::<Bip44<bip44::Account>>::EXTERNAL;
const CHANGE_INTERNAL: SoftDerivation = DerivationPath::<Bip44<bip44::Account>>::INTERNAL;
const DEFAULT_GAG_LIMIT: u32 = 20;

pub struct RecoveringIcarus {
    wallet: Wallet,
    accounts: Vec<RecoveringAccount>,
    value_total: Value,
    utxos: Vec<RecoveredUtxo>,
}

struct RecoveringAccount {
    id: HardDerivation,
    account: Account<XPrv>,
    next_index: SoftDerivation,
    soft_derivation_range_length: u32,
    addresses: HashMap<OldAddress, Address<XPrv>>,
}

struct RecoveredUtxo {
    pointer: UtxoPointer,
    key: Address<XPrv>,
}

impl RecoveringAccount {
    fn new(id: HardDerivation, account: Account<XPrv>) -> Self {
        let mut ra = Self {
            id,
            account,
            next_index: SoftDerivation::min_value(),
            soft_derivation_range_length: DEFAULT_GAG_LIMIT,
            addresses: HashMap::with_capacity(128),
        };

        ra.extend_range();

        ra
    }

    #[inline]
    fn lookup(&self, old_address: &OldAddress) -> Option<&Address<XPrv>> {
        self.addresses.get(old_address)
    }

    #[inline]
    fn within_last_range(&self, address: &Address<XPrv>) -> bool {
        let idx = address.path().address();
        let checked = idx.saturating_add(self.soft_derivation_range_length);

        checked >= self.next_index
    }

    fn extend_range(&mut self) {
        let start = self.next_index;
        let end = start.saturating_add(self.soft_derivation_range_length);
        self.next_index = end;

        let range = SoftDerivationRange::new(start..end);

        let internal_addresses = self.account.addresses(CHANGE_INTERNAL, range.clone());
        let external_addresses = self.account.addresses(CHANGE_EXTERNAL, range);

        let addresses = internal_addresses.chain(external_addresses);

        for address in addresses {
            let xpub = address.key().as_ref().public();
            let old_address = cardano_legacy_address::ExtendedAddr::new_simple(&xpub, None);
            let old_address = old_address.to_address();
            dbg!(old_address.to_string());
            self.addresses.insert(old_address, address);
        }
    }
}

impl RecoveringIcarus {
    pub(crate) fn new(wallet: Wallet) -> Self {
        let mut wallet = Self {
            wallet,
            accounts: Vec::new(),
            value_total: Value::zero(),
            utxos: Vec::with_capacity(128),
        };

        wallet.populate_first_account();

        wallet
    }

    pub fn value_total(&self) -> Value {
        self.value_total
    }

    fn populate_first_account(&mut self) {
        let account_id = HardDerivation::min_value();
        let account = self.wallet.create_account(account_id);

        let account = RecoveringAccount::new(account_id, account);
        self.accounts.push(account);
    }

    fn populate_new_account(&mut self) {
        let last_id = self
            .accounts
            .last()
            .expect("there is always one at least")
            .id;

        if let Some(id) = last_id.checked_add(1) {
            let account = self.wallet.create_account(id);
            let account = RecoveringAccount::new(id, account);
            self.accounts.push(account);
        } else {
            // DO NOTHING... we have reached 2^31 accounts already
        }
    }

    fn check_address(&mut self, address: &OldAddress) -> Option<Address<XPrv>> {
        let mut accounts = self.accounts.iter_mut();
        let mut result = None;

        while let Some(account) = accounts.next() {
            if let Some(address) = account.lookup(address).cloned() {
                if account.within_last_range(&address) {
                    account.extend_range();
                }

                result = Some(address);
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

    /// convenient function to parse a block and check for owned token
    pub fn check_blocks<'a>(&mut self, fragments: impl Iterator<Item = &'a Fragment>) {
        for fragment in fragments {
            let fragment_id = fragment.hash();
            if let Fragment::OldUtxoDeclaration(utxos) = fragment {
                for (output_index, (address, value)) in utxos.addrs.iter().enumerate() {
                    let pointer = UtxoPointer {
                        transaction_id: fragment_id,
                        output_index: output_index as u8,
                        value: *value,
                    };

                    self.check(pointer, address);
                }
            }
        }
    }

    pub fn check(&mut self, pointer: UtxoPointer, address: &OldAddress) {
        if let Some(key) = self.check_address(address) {
            self.value_total = self.value_total.saturating_add(pointer.value);
            self.utxos.push(RecoveredUtxo { pointer, key })
        }
    }

    /// dump all the inputs
    pub fn dump_in(&self, dump: &mut Dump) {
        for utxo in self.utxos.iter() {
            dump.push(
                Input::from_utxo(utxo.pointer),
                WitnessBuilder::OldUtxo {
                    xprv: utxo.key.key().as_ref().clone(),
                },
            )
        }
    }
}
