use crate::ByteArray;
use std::marker::PhantomData;
use std::num::NonZeroUsize;

/// A dynamically created buffer for T
#[derive(Clone, Default)]
pub struct ByteBuilder<T> {
    buffer: Vec<u8>,
    phantom: PhantomData<T>,
    expected: Option<NonZeroUsize>,
}

impl<T> From<ByteBuilder<T>> for Vec<u8> {
    fn from(bb: ByteBuilder<T>) -> Vec<u8> {
        bb.buffer
    }
}

impl<T> ByteBuilder<T> {
    /// Create an unconstrained Builder
    pub fn new() -> Self {
        ByteBuilder {
            buffer: Vec::new(),
            phantom: PhantomData,
            expected: None,
        }
    }

    /// Create a builder of fixed size
    pub fn new_fixed(size: NonZeroUsize) -> Self {
        ByteBuilder {
            buffer: Vec::with_capacity(size.get()),
            phantom: PhantomData,
            expected: Some(size),
        }
    }

    /// Append an u8 in the builder
    pub fn u8(self, v: u8) -> Self {
        let mut buf = self.buffer;
        buf.push(v);
        ByteBuilder {
            buffer: buf,
            phantom: self.phantom,
            expected: self.expected,
        }
    }
    /// Append bytes in the builder
    pub fn bytes(self, v: &[u8]) -> Self {
        let mut buf = self.buffer;
        buf.extend_from_slice(v);
        ByteBuilder {
            buffer: buf,
            phantom: self.phantom,
            expected: self.expected,
        }
    }

    /// fold over an iterator
    pub fn fold<F, I>(self, l: I, f: F) -> Self
    where
        I: Iterator,
        F: FnMut(Self, I::Item) -> Self,
    {
        l.fold(self, f)
    }

    /// Write an iterator of maximum 255 items using the closure `f`.
    ///
    /// Note that the buffer contains a byte to represent the size
    /// of the list.
    pub fn iter8<F, I>(self, l: I, f: F) -> Self
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        F: FnMut(Self, I::Item) -> Self,
    {
        let l = l.into_iter();
        let len = l.len();
        assert!(len <= u8::MAX as usize);
        let bb = self.u8(len as u8);
        l.fold(bb, f)
    }

    /// Write an iterator of maximum 2^16 - 1 items using the closure `f`.
    ///
    /// Note that the buffer contains 2 bytes to represent the size
    /// of the list.
    pub fn iter16<F, I>(self, l: I, f: F) -> Self
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        F: FnMut(Self, I::Item) -> Self,
    {
        let l = l.into_iter();
        let len = l.len();
        assert!(len <= u16::MAX as usize);
        let bb = self.u16(len as u16);
        l.fold(bb, f)
    }

    /// Write an iterator of maximum 2^32 - 1 items using the closure `f`.
    ///
    /// Note that the buffer contains 4 bytes to represent the size
    /// of the list.
    pub fn iter32<F, I>(self, l: I, f: F) -> Self
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        F: FnMut(Self, I::Item) -> Self,
    {
        let l = l.into_iter();
        let len = l.len();
        assert!(len <= u32::MAX as usize);
        let bb = self.u32(len as u32);
        l.fold(bb, f)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn sub<F, U>(self, f: F) -> Self
    where
        F: Fn(ByteBuilder<U>) -> ByteBuilder<U>,
    {
        let res = f(ByteBuilder {
            buffer: self.buffer,
            phantom: PhantomData,
            expected: None,
        });
        ByteBuilder {
            buffer: res.buffer,
            phantom: self.phantom,
            expected: self.expected,
        }
    }

    /// Append an u16 in the builder
    pub fn u16(self, v: u16) -> Self {
        self.bytes(&v.to_be_bytes())
    }

    /// Append an u32 in the builder
    pub fn u32(self, v: u32) -> Self {
        self.bytes(&v.to_be_bytes())
    }

    /// Append an u64 in the builder
    pub fn u64(self, v: u64) -> Self {
        self.bytes(&v.to_be_bytes())
    }

    /// Append an u128 in the builder
    pub fn u128(self, v: u128) -> Self {
        self.bytes(&v.to_be_bytes())
    }

    /// Finalize the buffer and return a fixed ByteArray of T
    pub fn finalize(self) -> ByteArray<T> {
        match self.expected {
            None => ByteArray::from_vec(self.buffer),
            Some(expected_sz) => {
                if expected_sz.get() == self.buffer.len() {
                    ByteArray::from_vec(self.buffer)
                } else {
                    panic!(
                        "internal-error: bytebuilder: expected size {} but got {}",
                        expected_sz.get(),
                        self.buffer.len()
                    )
                }
            }
        }
    }

    /// Finalize the buffer and return a fixed ByteArray of T
    pub fn finalize_as_vec(self) -> Vec<u8> {
        self.buffer
    }
}
