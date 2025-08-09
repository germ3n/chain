use {
    primitive_types::U256 as PrimitiveU256,
    bincode::{Encode, Decode},
    num_traits::ops::saturating::{SaturatingAdd, SaturatingSub},
    std::ops::{Add, Sub},
    std::cmp::{PartialOrd, Ordering},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Copy)]
pub struct U256(pub PrimitiveU256);

impl Encode for U256 {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.0.to_little_endian().encode(encoder)
    }
}

impl<Ctx> Decode<Ctx> for U256 {
    fn decode<D: bincode::de::Decoder<Context = Ctx>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let bytes: [u8; 32] = Decode::decode(decoder)?;
        Ok(U256(PrimitiveU256::from_little_endian(&bytes)))
    }
}

impl<'de, Ctx> bincode::BorrowDecode<'de, Ctx> for U256 {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Ctx>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Self::decode(decoder)
    }
}

impl Add for U256 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        U256(self.0 + other.0)
    }
}

impl Sub for U256 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        U256(self.0 - other.0)
    }
}

impl PartialOrd<u128> for U256 {
    fn partial_cmp(&self, other: &u128) -> Option<Ordering> {
        self.0.partial_cmp(&PrimitiveU256::from(*other))
    }
}

impl SaturatingAdd for U256 {
    fn saturating_add(&self, other: &Self) -> Self {
        U256(self.0.saturating_add(other.0))
    }
}

impl SaturatingSub for U256 {
    fn saturating_sub(&self, other: &Self) -> Self {
        U256(self.0.saturating_sub(other.0))
    }
}

impl PartialEq<u128> for U256 {
    fn eq(&self, other: &u128) -> bool {
        self.0 == PrimitiveU256::from(*other)
    }
}

impl From<u128> for U256 {
    fn from(value: u128) -> Self {
        U256(PrimitiveU256::from(value))
    }
}