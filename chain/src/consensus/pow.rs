use {
    crate::consensus::ConsensusLayerTrait,
    anyhow::Error,
    crate::crypto::hash::HashAlgorithm,
};

pub struct PoWConfig {
    pub hash_algorithm: HashAlgorithm,
}

pub struct PoW {
    pub config: Option<PoWConfig>,
}

impl ConsensusLayerTrait for PoW {
    fn initialize(&mut self) -> Result<(), Error> {
        self.config.replace(PoWConfig {
            hash_algorithm: HashAlgorithm::Blake3_256,
        });
        
        Ok(())
    }

    fn start_consensus(&self) -> Result<(), Error> {
        Ok(())
    }
}