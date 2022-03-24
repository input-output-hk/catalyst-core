use crate::value::Value;

pub use cardano_legacy_address::Addr as OldAddress;
pub use cardano_legacy_address::AddressMatchXPub as OldAddressMatchXPub;

use chain_core::property::WriteError;
use chain_core::{
    packer::Codec,
    property::{Deserialize, DeserializeFromSlice, ReadError, Serialize},
};
use chain_crypto::{Ed25519, PublicKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtxoDeclaration {
    pub addrs: Vec<(OldAddress, Value)>,
}

pub fn oldaddress_from_xpub(
    address: &OldAddress,
    pk: &PublicKey<Ed25519>,
    some_bytes: &[u8; 32],
) -> OldAddressMatchXPub {
    let mut pkraw = [0u8; 32];
    pkraw.copy_from_slice(pk.as_ref());
    address.identical_with_pubkey_raw(&pkraw, some_bytes)
}

impl DeserializeFromSlice for UtxoDeclaration {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let nb_entries = codec.get_u8()? as usize;
        if nb_entries >= 0xff {
            return Err(ReadError::StructureInvalid("nb entries".to_string()));
        }

        let mut addrs = Vec::with_capacity(nb_entries);
        for _ in 0..nb_entries {
            let value = Value::deserialize(codec)?;
            let addr_size = codec.get_be_u16()? as usize;
            let addr = OldAddress::try_from(codec.get_slice(addr_size)?)
                .map_err(|err| ReadError::StructureInvalid(format!("{}", err)))?;
            addrs.push((addr, value))
        }

        Ok(UtxoDeclaration { addrs })
    }
}

impl Serialize for UtxoDeclaration {
    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        assert!(self.addrs.len() < 255);

        codec.put_u8(self.addrs.len() as u8)?;
        for (b, v) in &self.addrs {
            v.serialize(codec)?;
            let bs = b.as_ref();
            codec.put_be_u16(bs.len() as u16)?;
            codec.put_bytes(bs)?;
        }
        Ok(())
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    use cardano_legacy_address::ExtendedAddr;
    use ed25519_bip32::{XPub, XPUB_SIZE};
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for UtxoDeclaration {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut nb: usize = Arbitrary::arbitrary(g);
            nb %= 255;
            let mut addrs = Vec::with_capacity(nb);
            for _ in 0..nb {
                let value = Arbitrary::arbitrary(g);

                let xpub = {
                    let mut buf = [0u8; XPUB_SIZE];
                    for o in buf.iter_mut() {
                        *o = u8::arbitrary(g)
                    }
                    match XPub::from_slice(&buf) {
                        Ok(xpub) => xpub,
                        Err(err) => panic!("xpub not built correctly, {:?}", err),
                    }
                };
                let ea = ExtendedAddr::new_simple(&xpub, None);
                let addr = ea.to_address();

                addrs.push((addr, value))
            }

            UtxoDeclaration { addrs }
        }
    }
}
