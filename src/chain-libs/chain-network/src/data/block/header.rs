/// A block header in the byte array representation.
#[derive(Clone, Debug)]
pub struct Header(Box<[u8]>);

impl Header {
    #[inline]
    pub fn from_bytes<B: Into<Box<[u8]>>>(bytes: B) -> Self {
        Header(bytes.into())
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into()
    }
}

impl AsRef<[u8]> for Header {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Header> for Vec<u8> {
    #[inline]
    fn from(header: Header) -> Self {
        header.into_bytes()
    }
}
