use {
    anyhow::Error,
    bincode::{Encode, Decode},
};

pub trait Serializable {
    fn to_bytes(&self) -> Result<Vec<u8>, Error>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error>
    where Self: Sized;
}

impl<T: Encode + Decode<()> + Sized> Serializable for T
{
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        bincode::encode_to_vec(self, bincode::config::standard()).map_err(anyhow::Error::msg)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let (value, _) = bincode::decode_from_slice(bytes, bincode::config::standard())?;
        Ok(value)
    }
}