use {
    crate::consensus::ConsensusLayerTrait,
    anyhow::Error,
    crate::crypto::hash::HashAlgorithm,
    crate::structs::block::Block,
    std::sync::atomic::Ordering,
    log::info,
    crate::ctx,
    cutedogs::document,
    crate::crypto::hash::Hashable,
    std::time::{SystemTime, UNIX_EPOCH},
    std::any::Any,
};

pub struct PoWConfig {
    pub hash_algorithm: HashAlgorithm,
}

pub struct PoW {
    pub config: Option<PoWConfig>,
}

impl ConsensusLayerTrait for PoW {
    #[document(summary = "Initializes the PoW consensus layer", returns = "The result of the initialization")]
    fn initialize(&mut self, data: &[&dyn Any]) -> Result<(), Error> {
        let hash_algorithm = data[0].downcast_ref::<HashAlgorithm>().unwrap_or(&HashAlgorithm::Blake3_256);
        self.config.replace(PoWConfig {
            hash_algorithm: hash_algorithm.clone(),
        });
        
        Ok(())
    }

    #[document(summary = "Creates a block for the PoW consensus layer", returns = "The created block")]
    fn create_block(&self) -> Result<Block, Error> {
        info!("Starting PoW consensus");
        info!("Mining block");
        let block_height = ctx!(block_height).load(Ordering::Relaxed);
        info!("Mining block height: {}", block_height);

        let hash_algorithm = self.config.as_ref().unwrap().hash_algorithm;

        let mut block = Block::coinbase(ctx!(curr_top).read().unwrap().clone(), Some(hash_algorithm));
        block.validate(Some(hash_algorithm))?;

        loop {
            block.header.nonce += 1;
            block.header.hash = block.hash(self.config.as_ref().unwrap().hash_algorithm);
            let zeros = u64::from_le_bytes(block.header.hash.get_bytes()[0..8].try_into().unwrap()).leading_zeros();
            if zeros >= ctx!(difficulty).load(Ordering::Relaxed) {
                break;
            }
        }

        Ok(block)
    }
}

impl PoW {
    #[document(summary = "Validates block timestamp against previous blocks", params = {
        "current_block": "The current block being validated",
        "previous_blocks": "A slice of previous blocks for timestamp validation"
    }, returns = "True if timestamp is valid")]
    pub fn validate_timestamp(&self, current_block: &Block, previous_blocks: &[Block]) -> bool {
        let current_time = current_block.header.timestamp;
        
        if previous_blocks.is_empty() {
            return true;
        }
        
        let median_time_past = self.calculate_median_time_past(previous_blocks);
        if current_time <= median_time_past {
            return false;
        }
        
        let network_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let max_future_time = 2 * 60 * 60 * 1000;
        if current_time > network_time + max_future_time {
            return false;
        }
        
        true
    }

    #[document(summary = "Calculates median time past from previous blocks", params = {
        "previous_blocks": "A slice of previous blocks"
    }, returns = "The median timestamp")]
    pub fn calculate_median_time_past(&self, previous_blocks: &[Block]) -> u64 {
        let blocks_to_consider = 11usize.min(previous_blocks.len());
        if blocks_to_consider == 0 {
            return 0;
        }
        
        let start_idx = previous_blocks.len().saturating_sub(blocks_to_consider);
        let mut timestamps: Vec<u64> = previous_blocks[start_idx..]
            .iter()
            .map(|block| block.header.timestamp)
            .collect();
        
        timestamps.sort_unstable();
        
        if timestamps.len() % 2 == 0 {
            let mid = timestamps.len() / 2;
            (timestamps[mid - 1] + timestamps[mid]) / 2
        } else {
            timestamps[timestamps.len() / 2]
        }
    }

    #[document(summary = "Calculates difficulty based on target block time with timestamp validation", params = {
        "current_block": "The current block being validated",
        "previous_blocks": "A slice of previous blocks for time calculation (should contain at least 1 block)"
    }, returns = "The adjusted difficulty")]
    pub fn calc_difficulty_by_target_time(&self, current_block: &Block, previous_blocks: &[Block]) -> u32 {
        let current_difficulty = ctx!(difficulty).load(Ordering::Relaxed);
        let target_time_ms = ctx!(config).target_ms_block_time as u64;
        
        if previous_blocks.is_empty() {
            return current_difficulty;
        }

        if !self.validate_timestamp(current_block, previous_blocks) {
            return current_difficulty.saturating_add(1);
        }

        let adjustment_interval = 10u64.min(previous_blocks.len() as u64);
        let blocks_to_analyze = &previous_blocks[previous_blocks.len().saturating_sub(adjustment_interval as usize)..];
        
        if blocks_to_analyze.len() < 2 {
            return current_difficulty;
        }

        let time_span = current_block.header.timestamp.saturating_sub(blocks_to_analyze[0].header.timestamp);
        let expected_time = target_time_ms * (blocks_to_analyze.len() as u64);
        
        let min_time_span = target_time_ms * blocks_to_analyze.len() as u64 / 4;
        if time_span == 0 || time_span < min_time_span {
            return current_difficulty.saturating_add(1);
        }

        let time_ratio = expected_time as f64 / time_span as f64;
        
        let max_adjustment = 4.0;
        let min_adjustment = 0.25;
        let clamped_ratio = time_ratio.max(min_adjustment).min(max_adjustment);
        
        let new_difficulty = (current_difficulty as f64 * clamped_ratio).round() as u32;
        
        let min_difficulty = 1u32;
        let max_difficulty = 32u32;
        
        new_difficulty.max(min_difficulty).min(max_difficulty)
    }
}