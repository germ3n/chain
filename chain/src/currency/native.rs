use {
    crate::currency::{CurrencyInfoTrait, CurrencyTrait, CurrencyInitTrait},
    crate::ctx,
    anyhow::Error,
    std::any::Any,
};

pub struct NativeCurrency {}

impl CurrencyInfoTrait for NativeCurrency {
    fn name(&self) -> &str {
        &ctx!(config).currency_name
    }

    fn symbol(&self) -> &str {
        &ctx!(config).currency_symbol
    }

    fn decimal_places(&self) -> u32 {
        ctx!(config).currency_decimal_places
    }

    fn one_currency_unit(&self) -> u128 {
        ctx!(config).one_currency_unit()
    }

    fn max_circulating_supply(&self) -> u128 {
        ctx!(config).max_circulating_supply
    }
}

impl CurrencyInitTrait for NativeCurrency {
    fn initialize(&self, _params: &[&dyn Any]) -> Result<(), Error> {
        Ok(())
    }
}

impl CurrencyTrait for NativeCurrency {
    fn mint(&self, amount: u128) -> Result<(), Error> {
        let mut current_supply = ctx!(issued_supply).write().unwrap();
        if amount + *current_supply > self.max_circulating_supply() {
            return Err(Error::msg("Max circulating supply exceeded"));
        }

        *current_supply = current_supply.saturating_add(amount);
        Ok(())
    }

    fn burn(&self, amount: u128) -> Result<(), Error> {
        let mut current_supply = ctx!(issued_supply).write().unwrap();
        if *current_supply < amount {
            return Err(Error::msg("Insufficient supply"));
        }

        *current_supply = current_supply.saturating_sub(amount);
        Ok(())
    }
}