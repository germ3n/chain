use {
    bincode::{Encode, Decode},
    serde::{Serialize, Deserialize},
    cutedogs::document,
};

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Config {
    pub max_circulating_supply: u128,
    pub emission_rate: u128,
    pub blocks_per_halving: u64,
    pub currency_name: String,
    pub currency_symbol: String,
    pub currency_decimal_places: u32,
}

impl Config {
    #[document(summary = "Calculates the one currency unit", returns = "The one currency unit")]
    pub fn one_currency_unit(&self) -> u128 {
        10_u128.pow(self.currency_decimal_places as u32)
    }

    #[cfg(feature = "halving-bitcoin")]
    #[document(summary = "Calculates the block reward based on the current block height", returns = "The calculated block reward")]
    pub fn calc_block_reward(&self, block_height: u64) -> u128 {
        match block_height / self.blocks_per_halving {
            64.. => 0,
            _ => self.emission_rate >> (block_height / self.blocks_per_halving),
        }
    }
}

impl Config {
    #[document(summary = "Creates a new bitcoin config", returns = "The new bitcoin config")]
    pub fn bitcoin() -> Self {
        let decimal_places = 8;
        let one_token = 10_u128.pow(decimal_places);
        Self { 
            max_circulating_supply: 21_000_000 * one_token,
            emission_rate: 50 * one_token,
            currency_decimal_places: decimal_places,
            blocks_per_halving: 210_000,
            currency_name: "Bitcoin".to_string(),
            currency_symbol: "BTC".to_string(),
        }
    }
}

impl std::default::Default for Config {
    fn default() -> Self {
        let decimal_places = 18;
        let one_token = 10_u128.pow(decimal_places);
        Self { 
            max_circulating_supply: one_token * 50_000_000,
            emission_rate: one_token * 100,
            currency_decimal_places: decimal_places,
            blocks_per_halving: 210_000,
            currency_name: "Native".to_string(),
            currency_symbol: "NAT".to_string(),
        }
    }
}