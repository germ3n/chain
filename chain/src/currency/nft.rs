use {
    crate::currency::{Amount, CurrencyInfoTrait, CurrencyTrait, CurrencyInitTrait},
    crate::structs::account::Address,
    anyhow::Error,
    std::any::Any,
    cutedogs::document,
};

pub struct NftCurrency {
    pub name: String,
    pub symbol: String,
}

impl CurrencyInfoTrait for NftCurrency {
    #[document(summary = "Get name of NFT currency", returns = "The name of the currency")]
    fn name(&self) -> &str {
        &self.name
    }

    #[document(summary = "Get symbol of NFT currency", returns = "The symbol of the currency")]
    fn symbol(&self) -> &str {
        &self.symbol
    }

    #[document(summary = "Get decimal places of NFT currency. Always returns 0.", returns = "The decimal places of the currency")]
    fn decimal_places(&self) -> u32 {
        0
    }
    
    #[document(summary = "Get one currency unit of NFT currency. Always returns 1.", returns = "The one currency unit of the currency")]
    fn one_currency_unit(&self) -> Amount {
        Amount::U128(1)
    }

    #[document(summary = "Get max circulating supply of NFT currency. Always returns 1.", returns = "The max circulating supply of the currency")]
    fn max_circulating_supply(&self) -> Amount {
        Amount::U128(1)
    }
}

impl CurrencyInitTrait for NftCurrency {
    #[document(summary = "Initialize NFT currency", returns = "The result of the initialization")]
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