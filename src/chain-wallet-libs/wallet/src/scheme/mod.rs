pub mod bip44;
pub mod freeutxo;
pub mod rindex;

use chain_impl_mockchain::{
    fragment::Fragment,
    transaction::{Input, Output, Witness},
};

pub(crate) fn on_tx_output<FO>(fragment: &Fragment, on_output: FO)
where
    FO: FnMut((usize, Output<chain_addr::Address>)),
{
    match fragment {
        Fragment::Initial(_config_params) => {}
        Fragment::UpdateProposal(_update_proposal) => {}
        Fragment::UpdateVote(_signed_update) => {}
        Fragment::OldUtxoDeclaration(_utxos) => {}
        Fragment::Transaction(tx) => tx
            .as_slice()
            .outputs()
            .iter()
            .enumerate()
            .for_each(on_output),
        Fragment::OwnerStakeDelegation(tx) => tx
            .as_slice()
            .outputs()
            .iter()
            .enumerate()
            .for_each(on_output),
        Fragment::StakeDelegation(tx) => tx
            .as_slice()
            .outputs()
            .iter()
            .enumerate()
            .for_each(on_output),
        Fragment::PoolRegistration(tx) => tx
            .as_slice()
            .outputs()
            .iter()
            .enumerate()
            .for_each(on_output),
        Fragment::PoolRetirement(tx) => tx
            .as_slice()
            .outputs()
            .iter()
            .enumerate()
            .for_each(on_output),
        Fragment::PoolUpdate(tx) => tx
            .as_slice()
            .outputs()
            .iter()
            .enumerate()
            .for_each(on_output),
        Fragment::VotePlan(tx) => tx
            .as_slice()
            .outputs()
            .iter()
            .enumerate()
            .for_each(on_output),
        Fragment::VoteCast(tx) => tx
            .as_slice()
            .outputs()
            .iter()
            .enumerate()
            .for_each(on_output),
        Fragment::VoteTally(tx) => tx
            .as_slice()
            .outputs()
            .iter()
            .enumerate()
            .for_each(on_output),
    }
}

pub(crate) fn on_tx_input<FI>(fragment: &Fragment, on_input: FI)
where
    FI: FnMut(Input),
{
    match fragment {
        Fragment::Initial(_config_params) => {}
        Fragment::UpdateProposal(_update_proposal) => {}
        Fragment::UpdateVote(_signed_update) => {}
        Fragment::OldUtxoDeclaration(_utxos) => {}
        Fragment::Transaction(tx) => tx.as_slice().inputs().iter().for_each(on_input),
        Fragment::OwnerStakeDelegation(tx) => tx.as_slice().inputs().iter().for_each(on_input),
        Fragment::StakeDelegation(tx) => tx.as_slice().inputs().iter().for_each(on_input),
        Fragment::PoolRegistration(tx) => tx.as_slice().inputs().iter().for_each(on_input),
        Fragment::PoolRetirement(tx) => tx.as_slice().inputs().iter().for_each(on_input),
        Fragment::PoolUpdate(tx) => tx.as_slice().inputs().iter().for_each(on_input),
        Fragment::VotePlan(tx) => tx.as_slice().inputs().iter().for_each(on_input),
        Fragment::VoteCast(tx) => tx.as_slice().inputs().iter().for_each(on_input),
        Fragment::VoteTally(tx) => tx.as_slice().inputs().iter().for_each(on_input),
    }
}

pub(crate) fn on_tx_input_and_witnesses<FI>(fragment: &Fragment, on_input: FI)
where
    FI: FnMut((Input, Witness)),
{
    match fragment {
        Fragment::Initial(_config_params) => {}
        Fragment::UpdateProposal(_update_proposal) => {}
        Fragment::UpdateVote(_signed_update) => {}
        Fragment::OldUtxoDeclaration(_utxos) => {}
        Fragment::Transaction(tx) => tx
            .as_slice()
            .inputs_and_witnesses()
            .iter()
            .for_each(on_input),
        Fragment::OwnerStakeDelegation(tx) => tx
            .as_slice()
            .inputs_and_witnesses()
            .iter()
            .for_each(on_input),
        Fragment::StakeDelegation(tx) => tx
            .as_slice()
            .inputs_and_witnesses()
            .iter()
            .for_each(on_input),
        Fragment::PoolRegistration(tx) => tx
            .as_slice()
            .inputs_and_witnesses()
            .iter()
            .for_each(on_input),
        Fragment::PoolRetirement(tx) => tx
            .as_slice()
            .inputs_and_witnesses()
            .iter()
            .for_each(on_input),
        Fragment::PoolUpdate(tx) => tx
            .as_slice()
            .inputs_and_witnesses()
            .iter()
            .for_each(on_input),
        Fragment::VotePlan(tx) => tx
            .as_slice()
            .inputs_and_witnesses()
            .iter()
            .for_each(on_input),
        Fragment::VoteCast(tx) => tx
            .as_slice()
            .inputs_and_witnesses()
            .iter()
            .for_each(on_input),
        Fragment::VoteTally(tx) => tx
            .as_slice()
            .inputs_and_witnesses()
            .iter()
            .for_each(on_input),
    }
}
