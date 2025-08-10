use {
    crate::consensus::ConsensusLayerTrait,
    anyhow::Error,
};

pub struct PoSConfig {
}

pub struct PoS {
}

impl ConsensusLayerTrait for PoS {
    fn initialize(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn start_consensus(&self) -> Result<(), Error> {
        Ok(())
    }
}