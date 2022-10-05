/// A gossip node in an opaque byte array representation.
#[derive(Clone)]
pub struct Node(Box<[u8]>);

pub type Nodes = Box<[Node]>;

impl Node {
    #[inline]
    pub fn from_bytes<B: Into<Box<[u8]>>>(bytes: B) -> Self {
        Node(bytes.into())
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

impl AsRef<[u8]> for Node {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Node> for Vec<u8> {
    #[inline]
    fn from(block: Node) -> Self {
        block.into_bytes()
    }
}

pub struct Gossip {
    pub nodes: Nodes,
}
