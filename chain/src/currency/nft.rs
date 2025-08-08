use {
    crate::currency::{CurrencyInfoTrait, CurrencyTrait, CurrencyInitTrait},
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
    
    fn one_token(&self) -> u128 {
        1
    }
}

impl CurrencyInitTrait for NftCurrency {
    fn initialize(&self, _params: &[&dyn Any]) -> Result<(), Error> {
        Ok(())
    }
}

impl CurrencyTrait for NftCurrency {
    fn mint(&self, _amount: u128) -> Result<(), Error> {
        Err(Error::msg("NFTs cannot be minted"))
    }

    fn burn(&self, _amount: u128) -> Result<(), Error> {
        Err(Error::msg("NFTs cannot be burned"))
    }
}