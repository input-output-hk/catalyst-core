/// A block fragment in the byte array representation.
#[derive(Clone)]
pub struct Fragment(Box<[u8]>);

impl Fragment {
    #[inline]
    pub fn from_bytes<B: Into<Box<[u8]>>>(bytes: B) -> Self {
        Fragment(bytes.into())
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

impl AsRef<[u8]> for Fragment {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Fragment> for Vec<u8> {
    #[inline]
    fn from(block: Fragment) -> Self {
        block.into_bytes()
    }
}
