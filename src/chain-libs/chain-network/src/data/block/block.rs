/// A block in the byte array representation.
#[derive(Clone)]
pub struct Block(Box<[u8]>);

impl Block {
    #[inline]
    pub fn from_bytes<B: Into<Box<[u8]>>>(bytes: B) -> Self {
        Block(bytes.into())
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

impl AsRef<[u8]> for Block {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Block> for Vec<u8> {
    #[inline]
    fn from(block: Block) -> Self {
        block.into_bytes()
    }
}
