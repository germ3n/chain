use {
    crate::currency::{Amount, CurrencyInfoTrait, CurrencyInitTrait, CurrencyTrait},
    anyhow::Error,
    std::any::Any,
    crate::structs::account::Address,
    crate::r#const::POWERS_OF_10,
    std::sync::{RwLock},
    num_traits::ops::saturating::{SaturatingAdd, SaturatingSub},
};

pub struct TokenCurrency {
    pub name: String,
    pub symbol: String,
    pub decimal_places: u32,
    pub contract_address: Address,
    pub issued_supply: RwLock<Amount>,
    pub max_circulating_supply: RwLock<Amount>,
}

impl TokenCurrency {
    pub fn new(name: String, symbol: String, decimal_places: u32, contract_address: Address, issued_supply: Amount, max_circulating_supply: Amount) -> Result<Self, Error> {
        if decimal_places > POWERS_OF_10.len() as u32 {
            return Err(Error::msg(format!("Decimal places must be less than {}", POWERS_OF_10.len())));
        }

        Ok(Self {
            name,
            symbol,
            decimal_places,
            contract_address,
            issued_supply: RwLock::new(match issued_supply {
                Amount::U128(val) => Amount::U128(val),
                Amount::U256(val) => Amount::U256(val),
            }),
            max_circulating_supply: RwLock::new(match max_circulating_supply {
                Amount::U128(val) => Amount::U128(val),
                Amount::U256(val) => Amount::U256(val),
            }),
        })
    }
}

impl CurrencyTrait for TokenCurrency {
    fn mint(&self, amount: &Amount) -> Result<(), Error> {
        match (amount, &*self.max_circulating_supply.read().unwrap(), &mut *self.issued_supply.write().unwrap()) {
            (Amount::U128(val), Amount::U128(max_circulating_supply), Amount::U128(issued_supply)) => {
                if *max_circulating_supply > 0 && *max_circulating_supply < issued_supply.saturating_add(*val) {
                    Err(Error::msg("Max circulating supply exceeded"))
                }
                else {
                    *issued_supply = issued_supply.saturating_add(*val);
                    Ok(())
                }
            }
            (Amount::U256(val), Amount::U256(max_circulating_supply), Amount::U256(issued_supply)) => {
                if *max_circulating_supply > 0 && *max_circulating_supply < issued_supply.saturating_add(val) {
                    Err(Error::msg("Max circulating supply exceeded"))
                }
                else {
                    *issued_supply = issued_supply.saturating_add(val);
                    Ok(())
                }
            }
            _ => Err(Error::msg("Unsupported amount type combination"))
        }
    }

    fn burn(&self, amount: &Amount) -> Result<(), Error> {
        match (amount, &*self.max_circulating_supply.read().unwrap(), &mut *self.issued_supply.write().unwrap()) {
            (Amount::U128(val), Amount::U128(max_circulating_supply), Amount::U128(issued_supply)) => {
                if *max_circulating_supply > 0 && *max_circulating_supply < issued_supply.saturating_add(*val) {
                    Err(Error::msg("Max circulating supply exceeded"))
                }
                else {
                    *issued_supply = issued_supply.saturating_sub(*val);
                    Ok(())
                }
            }
            (Amount::U256(val), Amount::U256(max_circulating_supply), Amount::U256(issued_supply)) => {
                if *max_circulating_supply > 0 && *max_circulating_supply < issued_supply.saturating_sub(val) {
                    Err(Error::msg("Max circulating supply exceeded"))
                }
                else {
                    *issued_supply = issued_supply.saturating_sub(val);
                    Ok(())
                }
            }
            _ => Err(Error::msg("Unsupported amount type combination"))
        }
    }

    fn can_transfer(&self, _from: &Address, _to: &Address, _amount: &Amount) -> Result<(), Error> {
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

    fn one_currency_unit(&self) -> Amount {
        Amount::U128(POWERS_OF_10[self.decimal_places as usize])
    }

    fn max_circulating_supply(&self) -> Amount {
        self.max_circulating_supply.read().unwrap().clone()
    }
}

impl CurrencyInitTrait for TokenCurrency {
    fn initialize(&self, _params: &[&dyn Any]) -> Result<(), Error> {
        Ok(())
    }
}