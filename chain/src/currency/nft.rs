use {
    crate::currency::{Amount, CurrencyInfoTrait, CurrencyTrait, CurrencyInitTrait},
    crate::structs::account::Address,
    anyhow::Error,
    std::any::Any,
};

pub struct NftCurrency {
    pub name: String,
    pub symbol: String,
}

impl CurrencyInfoTrait for NftCurrency {
    fn name(&self) -> &str {
        &self.name
    }

    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn decimal_places(&self) -> u32 {
        0
    }
    
    fn one_currency_unit(&self) -> Amount {
        Amount::U128(1)
    }

    fn max_circulating_supply(&self) -> Amount {
        Amount::U128(1)
    }
}

impl CurrencyInitTrait for NftCurrency {
    fn initialize(&self, _params: &[&dyn Any]) -> Result<(), Error> {
        Ok(())
    }
}

impl CurrencyTrait for NftCurrency {
    fn mint(&self, _amount: &Amount) -> Result<(), Error> {
        Err(Error::msg("NFTs cannot be minted"))
    }

    fn burn(&self, _amount: &Amount) -> Result<(), Error> {
        Err(Error::msg("NFTs cannot be burned"))
    }

    fn can_transfer(&self, _from: &Address, _to: &Address, _amount: &Amount) -> Result<(), Error> {
        Err(Error::msg("NFTs cannot be transferred"))
    }
}