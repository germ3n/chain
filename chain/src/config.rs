use {
    bincode::{Encode, Decode},
    serde::{Serialize, Deserialize},
};

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Config {
    pub max_circulating_supply: u128,
    pub emission_rate: u128,
    pub decimal_places: u32,
    pub blocks_per_halving: u64,
}

impl Config {
    pub fn one_token(&self) -> u128 {
        10_u128.pow(self.decimal_places as u32)
    }

    #[cfg(feature = "halving-bitcoin")]
    pub fn calc_block_reward(&self, block_height: u64) -> u128 {
        match block_height / self.blocks_per_halving {
            64.. => 0,
            _ => self.emission_rate >> (block_height / self.blocks_per_halving),
        }
    }
}

impl Config {
    pub fn bitcoin() -> Self {
        let decimal_places = 8;
        let one_token = 10_u128.pow(decimal_places);
        Self { 
            max_circulating_supply: 21_000_000 * one_token,
            emission_rate: 50 * one_token,
            decimal_places,
            blocks_per_halving: 210_000,
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
            decimal_places,
            blocks_per_halving: 210_000,
        }
    }
}