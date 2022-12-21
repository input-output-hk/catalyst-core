use crate::wallet_state::MainnetWalletState;
use crate::CardanoWallet;

/// Wallet state builder for Network state builder is a trait which creates nice interface for
/// defining role of particular mainnet wallet in voting round. Wallet can be a direct voter/ delegator
/// or representative
pub trait MainnetWalletStateBuilder {
    /// wallet registered as representative. This is simplification and wallet catalyst key is
    /// added to in memory list instead of going through public representative registration process
    fn as_representative(&self) -> MainnetWalletState;

    /// wallet registers as direct voter, meaning it will send self-delegation registration
    fn as_direct_voter(&self) -> MainnetWalletState;
    /// wallet registers as direct voter, meaning it will send self-delegation registration with
    /// given nonce = `slot_no`
    fn as_direct_voter_on_slot_no(&self, slot_no: u64) -> MainnetWalletState;
    /// wallet registers as delegator, meaning it will send delegation registration
    fn as_delegator(&self, delegations: Vec<(&CardanoWallet, u8)>) -> MainnetWalletState;
    /// wallet registers as delegator, meaning it will send delegation registration with
    /// given nonce = `slot_no`
    fn as_delegator_on_slot_no(
        &self,
        delegations: Vec<(&CardanoWallet, u8)>,
        slot_no: u64,
    ) -> MainnetWalletState;
}

impl MainnetWalletStateBuilder for CardanoWallet {
    fn as_representative(&self) -> MainnetWalletState {
        MainnetWalletState {
            rep: Some(self.catalyst_public_key()),
            registration_tx: None,
            stake: self.stake(),
            stake_address: self.reward_address().to_address(),
        }
    }

    fn as_direct_voter(&self) -> MainnetWalletState {
        self.as_direct_voter_on_slot_no(0)
    }

    fn as_direct_voter_on_slot_no(&self, slot_no: u64) -> MainnetWalletState {
        MainnetWalletState {
            rep: None,
            registration_tx: Some(self.generate_direct_voting_registration(slot_no)),
            stake: self.stake(),
            stake_address: self.reward_address().to_address(),
        }
    }

    fn as_delegator(&self, delegations: Vec<(&CardanoWallet, u8)>) -> MainnetWalletState {
        self.as_delegator_on_slot_no(delegations, 0)
    }

    fn as_delegator_on_slot_no(
        &self,
        delegations: Vec<(&CardanoWallet, u8)>,
        slot_no: u64,
    ) -> MainnetWalletState {
        MainnetWalletState {
            rep: None,
            registration_tx: Some(
                self.generate_delegated_voting_registration(
                    delegations
                        .into_iter()
                        .map(|(wallet, weight)| (wallet.catalyst_public_key(), u32::from(weight)))
                        .collect(),
                    slot_no,
                ),
            ),
            stake: self.stake(),
            stake_address: self.reward_address().to_address(),
        }
    }
}
