use super::Entry;
use super::pots;
use chain_ser::deser::{Deserialize, Serialize};
use chain_ser::packer::Codec;


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

    fn serialize<W: std::io::Write>(&self,  writer: W) -> Result<(), Self::Error> {
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
            }
            Entry::OldUtxo(entry) => {
                codec.put_u8(EntrySerializeCode::OldUtxo as u8)?;
            }
            Entry::Account((identifier, account_state)) => {
                codec.put_u8(EntrySerializeCode::Account as u8)?;
                // identifier.serialize(writer)?;
                // account_state.
            }
            Entry::ConfigParam(config_param) => {
                codec.put_u8(EntrySerializeCode::ConfigParam as u8)?;
                // config_param.serialize(writer)?;
            }
            Entry::UpdateProposal((proposal_id, proposal_state)) => {
                codec.put_u8(EntrySerializeCode::UpdateProposal as u8)?;
                // proposal_id.serialize(writer)?;
            }
            Entry::MultisigAccount((identifier, account_state)) => {
                codec.put_u8(EntrySerializeCode::MultisigAccount as u8)?;
                // identifier.serialize(writer)?;
            }
            Entry::MultisigDeclaration((identifier, declaration)) => {
                codec.put_u8(EntrySerializeCode::MultisigDeclaration as u8)?;
                // identifier.serialize(writer)?;
            }
            Entry::StakePool((pool_id, pool_state)) => {
                codec.put_u8(EntrySerializeCode::StakePool as u8)?;
            }
            Entry::LeaderParticipation((pool_id, participation)) => {
                codec.put_u8(EntrySerializeCode::LeaderParticipation as u8)?;
                // participation.serialize(writer)?;
            }
        }
        Ok(())
    }
}
// impl Serialize for Entry<'_> {
//     fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
//         S: Serializer {
//         match self {
//             Entry::Globals(entry) => {
//                 unimplemented!();
//                 // IoVec::
//                 // entry.writev(sys::iove)
//                 // serializer.serialize_bytes()
//             },
//             Entry::Pot(entry) => {
//                 unimplemented!();
//             },
//             Entry::Utxo(entry) => {
//                 unimplemented!();
//             },
//             Entry::OldUtxo(entry) => {
//                 unimplemented!();
//             },
//             Entry::Account((identifier, account_state)) => {
//                 unimplemented!();
//                 identifier.serialize()
//             },
//             Entry::ConfigParam(config_param) => {
//                 unimplemented!();
//             },
//             Entry::UpdateProposal((proposal_id, proposal_state)) => {
//
//             },
//             Entry::MultisigAccount((identifier, account_state)) => {
//                 unimplemented!();
//             },
//             Entry::MultisigDeclaration((identifier, declaration)) => {
//                 unimplemented!();
//             },
//             Entry::StakePool((pool_id, pool_state)) => {
//                 unimplemented!();
//             },
//             Entry::LeaderParticipation((pool_id, participation)) => {
//                 unimplemented!();
//             }
//         }
//         serializer.serialize_i8(0)
//     }
// }
