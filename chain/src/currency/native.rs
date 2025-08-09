use {
    crate::currency::{CurrencyInfoTrait, CurrencyTrait, CurrencyInitTrait},
    crate::ctx,
    anyhow::Error,
    std::any::Any,
    crate::structs::account::Address,
    crate::currency::Amount,
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

    fn one_currency_unit(&self) -> Amount {
        Amount::U128(ctx!(config).one_currency_unit())
    }

    fn max_circulating_supply(&self) -> Amount {
        Amount::U128(ctx!(config).max_circulating_supply)
    }
}

impl CurrencyInitTrait for NativeCurrency {
    fn initialize(&self, _params: &[&dyn Any]) -> Result<(), Error> {
        Ok(())
    }
}

impl CurrencyTrait for NativeCurrency {
    fn mint(&self, amount: &Amount) -> Result<(), Error> {
        match (amount, &self.max_circulating_supply()) {
            (Amount::U128(val), Amount::U128(max_circulating_supply)) => {
                if *max_circulating_supply > 0 && *max_circulating_supply < ctx!(issued_supply).read().unwrap().saturating_add(*val) {
                    Err(Error::msg("Max circulating supply exceeded"))
                }
                else {
                    Ok(())
                }
            }
            _ => return Err(Error::msg("Invalid amount")),
        }
    }

    fn burn(&self, amount: &Amount) -> Result<(), Error> {
        match amount {
            Amount::U128(val) => {
                let mut current_supply = ctx!(issued_supply).write().unwrap();
                if *current_supply < *val {
                    return Err(Error::msg("Insufficient supply"));
                }

                *current_supply = current_supply.saturating_sub(*val);
                Ok(())
            }
            _ => Err(Error::msg("Invalid amount")),
        }
    }

    fn can_transfer(&self, _from: &Address, _to: &Address, _amount: &Amount) -> Result<(), Error> {
        Ok(())
    }
}