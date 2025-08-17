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
};

pub struct PoWConfig {
    pub hash_algorithm: HashAlgorithm,
}

pub struct PoW {
    pub config: Option<PoWConfig>,
}

impl ConsensusLayerTrait for PoW {
    #[document(summary = "Initializes the PoW consensus layer", returns = "The result of the initialization")]
    fn initialize(&mut self) -> Result<(), Error> {
        self.config.replace(PoWConfig {
            hash_algorithm: HashAlgorithm::Blake3_256,
        });
        
        Ok(())
    }

    #[document(summary = "Creates a block for the PoW consensus layer", returns = "The created block")]
    fn create_block(&self) -> Result<Block, Error> {
        info!("Starting PoW consensus");
        info!("Mining block");
        let block_height = ctx!(block_height).load(Ordering::Relaxed);
        info!("Mining block height: {}", block_height);

        let mut block = Block::coinbase(ctx!(curr_top).read().unwrap().clone());
        block.validate()?;

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