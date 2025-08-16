use {
    crate::currency::{CurrencyInfoTrait, CurrencyTrait, CurrencyInitTrait},
    crate::ctx,
    anyhow::Error,
    std::any::Any,
    crate::structs::account::Address,
    crate::currency::Amount,
    cutedogs::document,
};

pub struct NativeCurrency {}

impl CurrencyInfoTrait for NativeCurrency {
    #[document(summary = "Get name of native currency", returns = "The name of the currency")]
    fn name(&self) -> &str {
        &ctx!(config).currency_name
    }

    #[document(summary = "Get symbol of native currency", returns = "The symbol of the currency")]
    fn symbol(&self) -> &str {
        &ctx!(config).currency_symbol
    }

    #[document(summary = "Get decimal places of native currency", returns = "The decimal places of the currency")]
    fn decimal_places(&self) -> u32 {
        ctx!(config).currency_decimal_places
    }

    #[document(summary = "Get one currency unit of native currency", returns = "The one currency unit of the currency")]
    fn one_currency_unit(&self) -> Amount {
        Amount::U128(ctx!(config).one_currency_unit())
    }

    #[document(summary = "Get max circulating supply of native currency", returns = "The max circulating supply of the currency")]
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
    #[document(summary = "Mint native currency", params = {
        "amount": "The amount to mint"
    }, returns = "The result of the minting")]
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

    #[document(summary = "Burn native currency", params = {
        "amount": "The amount to burn"
    }, returns = "The result of the burning")]
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

    #[document(summary = "Check if can transfer native currency", params = {
        "from": "The address of the sender",
        "to": "The address of the recipient",
        "amount": "The amount to transfer"
    }, returns = "The result of the transfer")]
    fn can_transfer(&self, _from: &Address, _to: &Address, _amount: &Amount) -> Result<(), Error> {
        Ok(())
    }
}