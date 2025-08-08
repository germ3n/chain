use {
    anyhow::Error,
    bincode::{Encode, Decode},
};

pub trait SerializableBytes {
    fn to_bytes(&self) -> Result<Vec<u8>, Error>;
}

pub trait DeserializableBytes {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error>
    where Self: Sized;
}

impl<T: Encode + Sized> SerializableBytes for T
{
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        bincode::encode_to_vec(self, bincode::config::standard()).map_err(Error::new)
    }
}

impl<T: Decode<()> + Sized> DeserializableBytes for T
{
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let (value, _) = bincode::decode_from_slice(bytes, bincode::config::standard())?;
        Ok(value)
    }
}

pub fn to_bytes<T: SerializableBytes + Sized>(value: &T) -> Result<Vec<u8>, Error> {
    value.to_bytes()
}

pub fn from_bytes<T: DeserializableBytes>(bytes: &[u8]) -> Result<T, Error> {
    T::from_bytes(bytes)
}