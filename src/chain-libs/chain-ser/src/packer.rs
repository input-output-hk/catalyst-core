//! Tooling for packing and unpacking from streams
//!
//! This will allow us to expose some standard way of serializing
//! data.

use crate::deser::{ReadError, WriteError};
use std::num::{NonZeroU32, NonZeroU64};

/// The structure to support (de)serialization of binary data format used by
/// jormungandr.
///
/// ## Reading data
///
/// The structure is generally intended to read from any implementor of
/// `std::io::Read`. On top of that it supports specialized methods for `&[u8]`
/// (which is also `std::io::Read`) to facilitate zero-copy operations.
///
/// ## Writing data
///
/// Data can be written into any `std::io::Write` implementor.
pub struct Codec<I> {
    inner: I,
}

impl<I> Codec<I> {
    pub fn new(inner: I) -> Self {
        Codec { inner }
    }

    pub fn into_inner(self) -> I {
        self.inner
    }
}

impl Codec<&[u8]> {
    #[inline]
    pub fn get_slice(&mut self, n: usize) -> Result<&[u8], ReadError> {
        if self.inner.as_ref().len() < n {
            return Err(ReadError::NotEnoughBytes(self.inner.as_ref().len(), n));
        }
        let res = &self.inner[..n];
        self.inner = &self.inner[n..];
        Ok(res)
    }

    #[inline]
    pub fn skip_bytes(&mut self, pos: usize) {
        self.inner = &self.inner[pos..];
    }

    #[inline]
    pub fn bytes_left(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn has_bytes_left(&self) -> bool {
        self.bytes_left() != 0
    }
}

impl<R: std::io::Read> Codec<R> {
    #[inline]
    pub fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, ReadError> {
        let res = self.inner.read_to_end(buf)?;
        Ok(res)
    }

    #[inline]
    pub fn get_u8(&mut self) -> Result<u8, ReadError> {
        let mut buf = [0u8; 1];
        self.inner.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    #[inline]
    pub fn get_be_u16(&mut self) -> Result<u16, ReadError> {
        let mut buf = [0u8; 2];
        self.inner.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    #[inline]
    pub fn get_le_u16(&mut self) -> Result<u16, ReadError> {
        let mut buf = [0u8; 2];
        self.inner.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    #[inline]
    pub fn get_be_u32(&mut self) -> Result<u32, ReadError> {
        let mut buf = [0u8; 4];
        self.inner.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    #[inline]
    pub fn get_le_u32(&mut self) -> Result<u32, ReadError> {
        let mut buf = [0u8; 4];
        self.inner.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    #[inline]
    pub fn get_be_u64(&mut self) -> Result<u64, ReadError> {
        let mut buf = [0u8; 8];
        self.inner.read_exact(&mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }

    #[inline]
    pub fn get_le_u64(&mut self) -> Result<u64, ReadError> {
        let mut buf = [0u8; 8];
        self.inner.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    #[inline]
    pub fn get_be_u128(&mut self) -> Result<u128, ReadError> {
        let mut buf = [0u8; 16];
        self.inner.read_exact(&mut buf)?;
        Ok(u128::from_be_bytes(buf))
    }

    #[inline]
    pub fn get_le_u128(&mut self) -> Result<u128, ReadError> {
        let mut buf = [0u8; 16];
        self.inner.read_exact(&mut buf)?;
        Ok(u128::from_le_bytes(buf))
    }

    #[inline]
    pub fn get_nz_u32(&mut self) -> Result<NonZeroU32, ReadError> {
        let val = self.get_be_u32()?;
        NonZeroU32::new(val)
            .ok_or_else(|| ReadError::StructureInvalid("received zero u32".to_string()))
    }

    #[inline]
    pub fn get_nz_u64(&mut self) -> Result<NonZeroU64, ReadError> {
        let val = self.get_be_u64()?;
        NonZeroU64::new(val)
            .ok_or_else(|| ReadError::StructureInvalid("received zero u64".to_string()))
    }

    #[inline]
    pub fn get_bytes(&mut self, n: usize) -> Result<Vec<u8>, ReadError> {
        let mut buf = vec![0u8; n];
        self.inner.read_exact(&mut buf)?;
        Ok(buf)
    }

    #[inline]
    pub fn copy_to_slice(&mut self, slice: &mut [u8]) -> Result<(), ReadError> {
        self.inner.read_exact(slice)?;
        Ok(())
    }
}

impl<W: std::io::Write> Codec<W> {
    #[inline]
    pub fn put_u8(&mut self, v: u8) -> Result<(), WriteError> {
        self.inner.write_all(&[v])?;
        Ok(())
    }

    #[inline]
    pub fn put_be_u16(&mut self, v: u16) -> Result<(), WriteError> {
        self.inner.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    #[inline]
    pub fn put_le_u16(&mut self, v: u16) -> Result<(), WriteError> {
        self.inner.write_all(&v.to_le_bytes())?;
        Ok(())
    }

    #[inline]
    pub fn put_be_u32(&mut self, v: u32) -> Result<(), WriteError> {
        self.inner.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    #[inline]
    pub fn put_le_u32(&mut self, v: u32) -> Result<(), WriteError> {
        self.inner.write_all(&v.to_le_bytes())?;
        Ok(())
    }

    #[inline]
    pub fn put_be_u64(&mut self, v: u64) -> Result<(), WriteError> {
        self.inner.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    #[inline]
    pub fn put_le_u64(&mut self, v: u64) -> Result<(), WriteError> {
        self.inner.write_all(&v.to_le_bytes())?;
        Ok(())
    }

    #[inline]
    pub fn put_be_u128(&mut self, v: u128) -> Result<(), WriteError> {
        self.inner.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    #[inline]
    pub fn put_le_u128(&mut self, v: u128) -> Result<(), WriteError> {
        self.inner.write_all(&v.to_le_bytes())?;
        Ok(())
    }

    #[inline]
    pub fn put_bytes(&mut self, v: &[u8]) -> Result<(), WriteError> {
        self.inner.write_all(v)?;
        Ok(())
    }
}

impl<T> Codec<std::io::Cursor<T>> {
    #[inline]
    pub fn position(&mut self) -> usize {
        self.inner.position() as usize
    }

    #[inline]
    pub fn set_position(&mut self, pos: usize) {
        self.inner.set_position(pos as u64)
    }
}
