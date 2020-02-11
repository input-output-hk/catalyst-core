/// Define that an object can be written to a `Write` object.
pub trait Serialize {
    type Error: std::error::Error + From<std::io::Error>;

    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error>;

    /// Convenience method to serialize into a byte vector.
    fn serialize_as_vec(&self) -> Result<Vec<u8>, Self::Error> {
        let mut data = vec![];
        self.serialize(&mut data)?;
        Ok(data)
    }
}

/// Define that an object can be read from a `Read` object.
pub trait Deserialize: Sized {
    type Error: std::error::Error + From<std::io::Error> + Send + Sync + 'static;

    fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error>;
}

impl<T: Serialize> Serialize for &T {
    type Error = T::Error;

    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), T::Error> {
        (**self).serialize(writer)
    }
}
