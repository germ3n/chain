pub mod native;
pub mod nft;
pub mod token;

use {
    anyhow::Error,
    std::any::Any,
    crate::structs::account::Address,
    crate::structs::num::U256,
    bincode::{Encode, Decode},
    serde::{Serialize, Deserialize},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[repr(u8)]
pub enum Amount {
    U128(u128) = 0,
    U256(U256) = 1,
}

pub trait CurrencyInfoTrait {
    fn name(&self) -> &str;
    fn symbol(&self) -> &str;
    fn decimal_places(&self) -> u32;
    fn one_currency_unit(&self) -> Amount;
    fn max_circulating_supply(&self) -> Amount;
}

pub trait CurrencyInitTrait {
    fn initialize(&self, params: &[&dyn Any]) -> Result<(), Error>;
}

pub trait CurrencyTrait: CurrencyInfoTrait + CurrencyInitTrait {
    fn mint(&self, amount: &Amount) -> Result<(), Error>;
    fn burn(&self, amount: &Amount) -> Result<(), Error>;
    fn can_transfer(&self, from: &Address, to: &Address, amount: &Amount) -> Result<(), Error>;
}

impl Encode for Amount {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        match self {
            Amount::U128(val) => (Amount::U128 as u8, val).encode(encoder),
            Amount::U256(val) => (Amount::U256 as u8, val).encode(encoder),
        }
    }
}

impl<Ctx> Decode<Ctx> for Amount 
where 
    U256: Decode<Ctx>,
{
    fn decode<D: bincode::de::Decoder<Context = Ctx>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let tag: u8 = Decode::decode(decoder)?;
        match tag {
            0/*Amount::U128*/ => Ok(Amount::U128(Decode::decode(decoder)?)),
            1/*Amount::U256*/ => Ok(Amount::U256(Decode::decode(decoder)?)),
            _ => Err(bincode::error::DecodeError::Other("Invalid tag")),
        }
    }
}

impl<'de, Ctx> bincode::BorrowDecode<'de, Ctx> for Amount 
where 
    U256: bincode::BorrowDecode<'de, Ctx>,
{
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Ctx>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Self::decode(decoder)
    }
}

impl std::fmt::Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Amount::U128(val) => write!(f, "{}", val),
            Amount::U256(_) => unimplemented!("U256 not supported"),
        }
    }
}