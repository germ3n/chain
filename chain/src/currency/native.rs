use {
    crate::currency::{CurrencyInfoTrait, CurrencyTrait, CurrencyInitTrait},
    crate::ctx,
    anyhow::Error,
    std::any::Any,
};

pub struct NativeCurrency {}

impl CurrencyInfoTrait for NativeCurrency {
    fn name(&self) -> &str {
        "Native"
    }

    fn symbol(&self) -> &str {
        "NAT"
    }

    fn decimal_places(&self) -> u32 {
        ctx!(config).decimal_places
    }

    fn one_token(&self) -> u128 {
        ctx!(config).one_token()
    }
}

impl CurrencyInitTrait for NativeCurrency {
    fn initialize(&self, _params: &[&dyn Any]) -> Result<(), Error> {
        Ok(())
    }
}

impl CurrencyTrait for NativeCurrency {
    fn mint(&self, _amount: u128) -> Result<(), Error> {
        if _amount + *ctx!(issued_supply).read().unwrap() > ctx!(config).max_circulating_supply {
            Err(Error::msg("Max circulating supply exceeded"))
        } else {
            Ok(())
        }
    }

    fn burn(&self, _amount: u128) -> Result<(), Error> {
        Ok(())
    }
}