use super::pots;
use super::{Entry, EntryOwned};
use chain_addr::Address;
use chain_ser::deser::{Deserialize, Serialize};
use chain_ser::packer::Codec;
use std::io::{Error, Write};
use std::iter::FromIterator;
use crate::ledger::Ledger;
use crate::ledger::iter;
use crate::ledger::pots::EntryType;

#[derive(Debug, Eq, PartialEq)]
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
    SerializationEnd = 11,
}

impl EntrySerializeCode {
    pub fn from_u8(n: u8) -> Option<Self> {
        match n {
            0 => Some(EntrySerializeCode::Globals),
            1 => Some(EntrySerializeCode::Pot),
            2 => Some(EntrySerializeCode::Utxo),
            3 => Some(EntrySerializeCode::OldUtxo),
            4 => Some(EntrySerializeCode::Account),
            5 => Some(EntrySerializeCode::ConfigParam),
            6 => Some(EntrySerializeCode::UpdateProposal),
            7 => Some(EntrySerializeCode::MultisigAccount),
            8 => Some(EntrySerializeCode::MultisigDeclaration),
            9 => Some(EntrySerializeCode::StakePool),
            10 => Some(EntrySerializeCode::LeaderParticipation),
            11 => Some(EntrySerializeCode::SerializationEnd),
            _ => None,
        }
    }
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

impl Deserialize for EntryOwned {
    type Error = std::io::Error;

    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
        let mut codec = Codec::new(reader);
        let code_u8 = codec.get_u8()?;
        let code = EntrySerializeCode::from_u8(code_u8).ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Error reading Entry, not recognized type code {}", code_u8),
        ))?;
        match code {
            EntrySerializeCode::Globals => Ok(EntryOwned::Globals(super::Globals::deserialize(
                &mut codec,
            )?)),
            EntrySerializeCode::Pot => Ok(EntryOwned::Pot(super::pots::Entry::deserialize(
                &mut codec,
            )?)),
            EntrySerializeCode::Utxo => Ok(EntryOwned::Utxo(crate::utxo::EntryOwned::deserialize(
                &mut codec,
            )?)),
            EntrySerializeCode::OldUtxo => Ok(EntryOwned::OldUtxo(
                crate::utxo::EntryOwned::deserialize(&mut codec)?,
            )),
            EntrySerializeCode::Account => {
                let identifier = crate::account::Identifier::deserialize(&mut codec)?;
                let account = crate::accounting::account::AccountState::deserialize(&mut codec)?;
                Ok(EntryOwned::Account((identifier, account)))
            }
            EntrySerializeCode::ConfigParam => Ok(EntryOwned::ConfigParam(
                crate::config::ConfigParam::deserialize(&mut codec)?,
            )),
            EntrySerializeCode::UpdateProposal => {
                let proposal_id = crate::update::UpdateProposalId::deserialize(&mut codec)?;
                let proposal_state = crate::update::UpdateProposalState::deserialize(&mut codec)?;
                Ok(EntryOwned::UpdateProposal((proposal_id, proposal_state)))
            },
            EntrySerializeCode::MultisigAccount => {
                let identifier = crate::multisig::Identifier::deserialize(&mut codec)?;
                let account_state =
                    crate::accounting::account::AccountState::deserialize(&mut codec)?;
                Ok(EntryOwned::MultisigAccount((identifier, account_state)))
            },
            EntrySerializeCode::MultisigDeclaration => {
                let identifier = crate::multisig::Identifier::deserialize(&mut codec)?;
                let declaration = crate::multisig::Declaration::deserialize(&mut codec)?;
                Ok(EntryOwned::MultisigDeclaration((identifier, declaration)))
            },
            EntrySerializeCode::StakePool => {
                let pool_id = crate::certificate::PoolId::deserialize(&mut codec)?;
                let pool_state = crate::stake::PoolState::deserialize(&mut codec)?;
                Ok(EntryOwned::StakePool((pool_id, pool_state)))
            },
            EntrySerializeCode::LeaderParticipation => {
                let pool_id = crate::certificate::PoolId::deserialize(&mut codec)?;
                let v = codec.get_u32()?;
                Ok(EntryOwned::LeaderParticipation((pool_id, v)))
            },
            EntrySerializeCode::SerializationEnd => {
                Ok(EntryOwned::StopEntry)
            }
        }
    }
}

impl Serialize for Ledger {
    type Error = std::io::Error;

    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        let mut codec = Codec::new(writer);
        for entry in self.iter() {
            entry.serialize(&mut codec)?;
        }
        // Write finish flag
        codec.put_u8(EntrySerializeCode::SerializationEnd as u8)?;
        Ok(())
    }
}

// struct LazyLedgerDeserializer<'a> {
//     reader: &'a mut dyn std::io::BufRead,
// }
//
// impl<'a> LazyLedgerDeserializer<'a> {
//     fn new<R: std::io::BufRead>(reader: &'a mut R) -> LazyLedgerDeserializer<'a> {
//         Self {
//             reader,
//         }
//     }
//
//     fn next(&mut self) -> Option<EntryOwned> {
//         // TODO: What to do with an error here?
//         EntryOwned::deserialize(&mut self.reader).ok()
//     }
// }
//
// impl<'a> IntoIterator for &'a mut LazyLedgerDeserializer<'a> {
//     type Item = EntryOwned;
//     type IntoIter = LazyLedgerDeserializerIter<'a>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         LazyLedgerDeserializerIter {
//             inner: self,
//         }
//     }
// }
//
// struct LazyLedgerDeserializerIter<'a> {
//     inner: &'a mut LazyLedgerDeserializer<'a>
// }
//
// impl<'a> Iterator for LazyLedgerDeserializerIter<'a> {
//     type Item = EntryOwned;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         self.inner.next()
//     }
// }


// impl Deserialize for Ledger {
//     type Error = std::io::Error;
//
//     fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
//         let mut codec = Codec::new(reader);
//         let res = Ok(Ledger::empty())::from_iter(
//             LazyLedgerDeserializer::new(&mut codec).into_iter().map(|entry| entry.to_entry())
//         );
//         match res {
//             Ok(ledger) => Ok(ledger),
//             Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{:?}", e))),
//         }
//     }
// }


#[cfg(test)]
pub mod test {
    #[test]
    #[ignore]
    fn ledger_serialize_deserialize_bijection() {
        unimplemented!()
    }
}
