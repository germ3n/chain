use {
    bincode::{Encode, Decode},
    serde::{Serialize, Deserialize},
    cutedogs::document,
    crate::r#const::POWERS_OF_10,
};

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Config {
    pub max_circulating_supply: u128,
    pub emission_rate: u128,
    pub blocks_per_halving: u64,
    pub currency_name: String,
    pub currency_symbol: String,
    pub currency_decimal_places: u32,
    pub target_ms_block_time: u32,
}

impl Config {
    #[document(summary = "Calculates the one currency unit", returns = "The one currency unit")]
    pub fn one_currency_unit(&self) -> u128 {
        POWERS_OF_10[self.currency_decimal_places as usize]
    }

    #[cfg(feature = "halving-bitcoin")]
    #[document(summary = "Calculates the block reward based on the current block height", returns = "The calculated block reward")]
    pub fn calc_block_reward(&self, block_height: u64) -> u128 {
        // The right bit-shift calculates the reward halving.
        // A halving occurs every `blocks_per_halving`.
        // After 64 halvings, the reward becomes 0.
        match block_height / self.blocks_per_halving {
            64.. => 0,
            halvings => self.emission_rate >> halvings,
        }
    }
}

/// A macro to generate a new currency configuration function.
macro_rules! create_config {
    (
        $name:ident,
        $summary:expr,
        {
            max_circulating_supply: $max_circulating_supply:expr,
            emission_rate: $emission_rate:expr,
            decimal_places: $decimal_places:expr,
            blocks_per_halving: $blocks_per_halving:expr,
            currency_name: $currency_name:expr,
            currency_symbol: $currency_symbol:expr,
            target_ms_block_time: $target_ms_block_time:expr $(,)* // Optional trailing comma
        }
    ) => {
        #[document(summary = $summary, returns = "The new config")]
        pub fn $name() -> Self {
            let decimal_places = $decimal_places;
            let one_token = POWERS_OF_10[decimal_places as usize];
            Self {
                max_circulating_supply: $max_circulating_supply * one_token,
                emission_rate: $emission_rate * one_token,
                currency_decimal_places: decimal_places,
                blocks_per_halving: $blocks_per_halving,
                currency_name: $currency_name.to_string(),
                currency_symbol: $currency_symbol.to_string(),
                target_ms_block_time: $target_ms_block_time,
            }
        }
    };
}

impl Config {
    create_config!(
        bitcoin,
        "Creates a new Bitcoin config.",
        {
            max_circulating_supply: 21_000_000,
            emission_rate: 50,
            decimal_places: 8,
            blocks_per_halving: 210_000,
            currency_name: "Bitcoin",
            currency_symbol: "BTC",
            target_ms_block_time: 1_000 * 60 * 10,
        }
    );

    create_config!(
        native,
        "Creates the native/default config.",
        {
            max_circulating_supply: 50_000_000,
            emission_rate: 100,
            decimal_places: 18,
            blocks_per_halving: 50_0000,
            currency_name: "Native",
            currency_symbol: "NAT",
            target_ms_block_time: 20_000,
        }
    );

    // Here's an example of how easy it is to add another configuration.
    create_config!(
        dogecoin,
        "Creates a new Dogecoin-like config.",
        {
            max_circulating_supply: 200_000_000_000, // Example large supply
            emission_rate: 10_000,
            decimal_places: 8,
            blocks_per_halving: 100_000,
            currency_name: "Dogecoin",
            currency_symbol: "DOGE",
            target_ms_block_time: 1_000 * 60, // 1 minute block time
        }
    );
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self::native()
    }
}