mod utils;

use chain_impl_mockchain::block::Block;
use chain_ser::mempack::Readable;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Wallet {
    daedalus: wallet::RecoveringDaedalus,
    icarus: wallet::RecoveringIcarus,
    account: wallet::Wallet,
}

#[wasm_bindgen]
pub struct Settings(wallet::Settings);

#[wasm_bindgen]
impl Wallet {
    /// retrieve a wallet from the given mnemonics and password
    ///
    /// this function will work for all yoroi, daedalus and other wallets
    /// as it will try every kind of wallet anyway
    ///
    /// You can also use this function to recover a wallet even after you have
    /// transferred all the funds to the new format (see the _convert_ function)
    ///
    ///
    /// the mnemonics should be in english
    pub fn recover(mnemonics: &str, password: &str) -> Result<Wallet, JsValue> {
        let builder = wallet::RecoveryBuilder::new();
        // TODO: recover from more languages?
        let builder = if let Ok(builder) = builder.mnemonics(&bip39::dictionary::ENGLISH, mnemonics)
        {
            builder
        } else {
            return Err(JsValue::from("invalid mnemonics"));
        };

        let builder = if password.len() > 0 { todo!() } else { builder };

        // calling this function cannot fail, we already
        // have the mnemonics set in the builder, and there is no password set
        let daedalus = builder
            .build_daedalus()
            .expect("build the daedalus wallet as expected");

        // calling this function cannot fail, we already
        // have the mnemonics set in the builder, and there is no password set
        let icarus = builder
            .build_yoroi()
            .expect("build the daedalus wallet as expected");

        // calling this function cannot fail as we have set the mnemonics already
        // and no password is valid (though it is weak security from daedalus wallet PoV)
        let account = builder
            .build_wallet()
            .expect("build the account cannot fail as expected");

        Ok(Wallet {
            account,
            daedalus,
            icarus,
        })
    }

    /// retrieve funds from daedalus or yoroi wallet in the given block0 (or
    /// any other blocks).
    ///
    /// Execute this function then you can check who much funds you have
    /// retrieved from the given block.
    ///
    /// this function may take sometimes so it is better to only call this
    /// function if needed.
    ///
    /// also, this function should not be called twice with the same block.
    pub fn retrieve_funds(&mut self, block0: &[u8]) -> Result<Settings, JsValue> {
        let mut bufreader = chain_ser::mempack::ReadBuf::from(block0);
        let block0 = if let Ok(block) = Block::read(&mut bufreader) {
            block
        } else {
            return Err(JsValue::from("invalid block0 format"));
        };

        let settings = Settings(wallet::Settings::new(&block0).unwrap());

        self.daedalus.check_blocks(block0.contents.iter());
        self.icarus.check_blocks(block0.contents.iter());

        Ok(settings)
    }

    /// get the total value in the wallet
    ///
    /// make sure to call `retrieve_funds` prior to calling this function
    /// otherwise you will always have `0`
    pub fn total_value(&self) -> u64 {
        self.icarus
            .value_total()
            .saturating_add(self.daedalus.value_total())
            .0
    }
}
