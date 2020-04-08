use crate::recovering::{dump::WitnessBuilder, Dump};
use chain_impl_mockchain::{
    legacy::OldAddress,
    transaction::{Input, UtxoPointer},
    value::Value,
};
use chain_path_derivation::rindex::{self, Rindex};
use ed25519_bip32::XPrv;
use hdkeygen::{
    rindex::{AddressRecovering, Wallet},
    Key,
};

pub struct RecoveringDaedalus {
    wallet: Wallet,
    address_recovering: AddressRecovering,
    value_total: Value,
    utxos: Vec<RecoveredUtxo>,
}

struct RecoveredUtxo {
    pointer: UtxoPointer,
    key: Key<XPrv, Rindex<rindex::Address>>,
}

impl RecoveringDaedalus {
    pub(crate) fn new(wallet: Wallet) -> Self {
        let address_recovering = wallet.address_recovering();
        Self {
            wallet,
            address_recovering,
            value_total: Value::zero(),
            utxos: Vec::with_capacity(128),
        }
    }

    pub fn value_total(&self) -> Value {
        self.value_total
    }

    pub fn check(&mut self, pointer: UtxoPointer, address: &OldAddress) {
        if let Some(derivation_path) = self.address_recovering.check_address(address) {
            self.value_total = self.value_total.saturating_add(pointer.value);
            let key = self.wallet.key(&derivation_path);
            self.utxos.push(RecoveredUtxo { pointer, key })
        }
    }

    pub fn dump_in(&self, dump: &mut Dump) {
        for utxo in self.utxos.iter() {
            dump.push(
                Input::from_utxo(utxo.pointer),
                WitnessBuilder::OldUtxo {
                    xprv: utxo.key.clone(),
                },
            )
        }
    }
}
