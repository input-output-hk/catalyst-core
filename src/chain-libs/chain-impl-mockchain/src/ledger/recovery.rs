use super::pots;
use super::Entry;
use chain_ser::deser::{Deserialize, Serialize};
use chain_ser::packer::Codec;
use std::io::Error;

#[derive(Eq, PartialEq)]
enum EntrySerializeCode {
    Globals = 0,
    Pot = 1,
    Utxo = 2,
    OldUtxo = 3,
    Account = 4,
    ConfigParam = 5,
    UpdateProposal = 6,
    MultisigAccount = 7,
    MultisigDeclaration = 8,
    StakePool = 9,
    LeaderParticipation = 10,
}

impl Serialize for Entry<'_> {
    type Error = std::io::Error;

    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        let mut codec = Codec::new(writer);
        match self {
            Entry::Globals(entry) => {
                codec.put_u8(EntrySerializeCode::Globals as u8)?;
                entry.serialize(&mut codec)?;
            }
            Entry::Pot(entry) => {
                codec.put_u8(EntrySerializeCode::Pot as u8)?;
                entry.serialize(&mut codec)?;
            }
            Entry::Utxo(entry) => {
                codec.put_u8(EntrySerializeCode::Utxo as u8)?;
                entry.serialize(&mut codec)?;
            }
            Entry::OldUtxo(entry) => {
                codec.put_u8(EntrySerializeCode::OldUtxo as u8)?;
                entry.serialize(&mut codec)?;
            }
            Entry::Account((ref identifier, ref account_state)) => {
                codec.put_u8(EntrySerializeCode::Account as u8)?;
                identifier.serialize(&mut codec)?;
                account_state.serialize(&mut codec)?;
            }
            Entry::ConfigParam(config_param) => {
                codec.put_u8(EntrySerializeCode::ConfigParam as u8)?;
                config_param.serialize(&mut codec)?;
            }
            Entry::UpdateProposal((proposal_id, proposal_state)) => {
                codec.put_u8(EntrySerializeCode::UpdateProposal as u8)?;
                proposal_id.serialize(&mut codec)?;
                proposal_state.serialize(&mut codec)?;
            }
            Entry::MultisigAccount((identifier, account_state)) => {
                codec.put_u8(EntrySerializeCode::MultisigAccount as u8)?;
                identifier.serialize(&mut codec)?;
                account_state.serialize(&mut codec)?;
            }
            Entry::MultisigDeclaration((identifier, declaration)) => {
                codec.put_u8(EntrySerializeCode::MultisigDeclaration as u8)?;
                identifier.serialize(&mut codec)?;
                declaration.serialize(&mut codec)?;
            }
            Entry::StakePool((pool_id, pool_state)) => {
                codec.put_u8(EntrySerializeCode::StakePool as u8)?;
                pool_id.serialize(&mut codec)?;
                pool_state.serialize(&mut codec)?;
            }
            Entry::LeaderParticipation((pool_id, participation)) => {
                codec.put_u8(EntrySerializeCode::LeaderParticipation as u8)?;
                pool_id.serialize(&mut codec)?;
                codec.put_u32(**participation)?;
            }
        }
        Ok(())
    }
}
