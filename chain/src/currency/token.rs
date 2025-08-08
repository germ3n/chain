use {
    crate::currency::{CurrencyInfoTrait, CurrencyInitTrait, CurrencyTrait},
    anyhow::Error,
    std::any::Any,
    crate::structs::account::Address,
    crate::config::POWERS_OF_10,
};

pub struct TokenCurrency {
    pub name: String,
    pub symbol: String,
    pub decimal_places: u32,
    pub contract_address: Address,
    pub max_circulating_supply: u128,
}

impl TokenCurrency {
    pub fn new(name: String, symbol: String, decimal_places: u32, contract_address: Address, max_circulating_supply: u128) -> Result<Self, Error> {
        if decimal_places > POWERS_OF_10.len() as u32 {
            return Err(Error::msg(format!("Decimal places must be less than {}", POWERS_OF_10.len())));
        }

        Ok(Self {
            name,
            symbol,
            decimal_places,
            contract_address,
            max_circulating_supply,
        })
    }
}

impl CurrencyTrait for TokenCurrency {
    fn mint(&self, _amount: u128) -> Result<(), Error> {
        Ok(())
    }

    fn burn(&self, _amount: u128) -> Result<(), Error> {
        Ok(())
    }
}

impl CurrencyInfoTrait for TokenCurrency {
    fn name(&self) -> &str {
        &self.name
    }

    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn decimal_places(&self) -> u32 {
        self.decimal_places
    }

    fn one_currency_unit(&self) -> u128 {
        POWERS_OF_10[self.decimal_places as usize]
    }

    fn max_circulating_supply(&self) -> u128 {
        self.max_circulating_supply
    }
}

impl CurrencyInitTrait for TokenCurrency {
    fn initialize(&self, _params: &[&dyn Any]) -> Result<(), Error> {
        Ok(())
    }
}