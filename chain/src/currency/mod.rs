pub mod native;
pub mod nft;

use {
    anyhow::Error,
    std::any::Any,
};

pub trait CurrencyInfoTrait {
    fn name(&self) -> &str;
    fn symbol(&self) -> &str;
    fn decimal_places(&self) -> u32;
    fn one_token(&self) -> u128;
}

pub trait CurrencyInitTrait {
    fn initialize(&self, params: &[&dyn Any]) -> Result<(), Error>;
}

pub trait CurrencyTrait: CurrencyInfoTrait + CurrencyInitTrait {
    fn mint(&self, amount: u128) -> Result<(), Error>;
    fn burn(&self, amount: u128) -> Result<(), Error>;
}