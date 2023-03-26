use crate::{
    validation::{cbor::cbor_to_bytes, hash::hash},
    Sig, Signature, VotingKey, VotingPurpose,
};

use super::{
    Nonce, PubKey, Registration, RewardsAddress, SignedRegistration, StakeKeyHex, TxId,
    VotingKeyHex,
};
use cardano_serialization_lib::chain_crypto::{AsymmetricKey, Ed25519, SigningAlgorithm};
use proptest::{arbitrary::StrategyFor, prelude::*, strategy::Map};

type Inputs = (Nonce, RewardsAddress, VotingKeyHex, [u8; 32], TxId);

impl Arbitrary for SignedRegistration {
    type Parameters = ();
    type Strategy = Map<StrategyFor<Inputs>, fn(Inputs) -> Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        any::<Inputs>().prop_map(
            |(nonce, rewards_address, voting_key, stake_secret, tx_id)| {
                arbitrary_signed_registration(
                    nonce,
                    rewards_address,
                    voting_key,
                    stake_secret,
                    tx_id,
                )
            },
        )
    }
}

pub(crate) fn arbitrary_signed_registration(
    nonce: Nonce,
    rewards_address: RewardsAddress,
    voting_key: VotingKeyHex,
    stake_secret: [u8; 32],
    tx_id: TxId,
) -> SignedRegistration {
    let secret = Ed25519::secret_from_binary(&stake_secret).unwrap();
    let public = Ed25519::compute_public(&secret);

    let stake_key: [u8; 32] = public.as_ref().try_into().unwrap();

    let registration = Registration {
        voting_key: VotingKey::Direct(voting_key),
        stake_key: StakeKeyHex(PubKey(stake_key.to_vec())),
        rewards_address,
        nonce,
        voting_purpose: Some(VotingPurpose::CATALYST),
    };

    let reg_cbor = registration.to_cbor();
    let reg_bytes = cbor_to_bytes(&reg_cbor);
    let reg_hash = hash(&reg_bytes);

    let sig = Ed25519::sign(&secret, &reg_hash);
    let sig_bytes: [u8; 64] = sig.as_ref().try_into().unwrap();

    let signature = Signature {
        inner: Sig(sig_bytes),
    };

    SignedRegistration {
        registration,
        signature,
        tx_id,
        slot: 12345,
        staked_ada: None,
    }
}

impl Arbitrary for RewardsAddress {
    type Parameters = ();
    type Strategy = Map<StrategyFor<Vec<u8>>, fn(Vec<u8>) -> Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        any::<Vec<u8>>().prop_map(|vec| RewardsAddress(vec.into()))
    }
}
