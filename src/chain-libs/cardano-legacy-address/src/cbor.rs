//! the CBOR util and compatible with the haskell usage...

pub mod util {
    //! CBor util and other stuff

    use crate::crc32::crc32;
    use cbor_event::{self, de::Deserializer, se::Serializer, Len};

    pub fn encode_with_crc32_<T, W>(t: &T, s: &mut Serializer<W>) -> cbor_event::Result<()>
    where
        T: cbor_event::Serialize,
        W: ::std::io::Write + Sized,
    {
        let bytes = cbor!(t)?;
        let crc32 = crc32(&bytes);
        s.write_array(Len::Len(2))?
            .write_tag(24)?
            .write_bytes(&bytes)?
            .write_unsigned_integer(crc32 as u64)?;
        Ok(())
    }

    pub fn raw_with_crc32<R: std::io::BufRead>(
        raw: &mut Deserializer<R>,
    ) -> cbor_event::Result<Vec<u8>> {
        let len = raw.array()?;
        assert!(len == Len::Len(2));

        let tag = raw.tag()?;
        if tag != 24 {
            return Err(cbor_event::Error::CustomError(format!(
                "Invalid Tag: {} but expected 24",
                tag
            )));
        }
        let bytes = raw.bytes()?;

        let crc = raw.unsigned_integer()?;

        let found_crc = crc32(&bytes);

        if crc != found_crc as u64 {
            return Err(cbor_event::Error::CustomError(format!(
                "Invalid CRC32: 0x{:x} but expected 0x{:x}",
                crc, found_crc
            )));
        }

        Ok(bytes)
    }
}
