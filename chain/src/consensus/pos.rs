use {
    crate::consensus::ConsensusLayerTrait,
    anyhow::Error,
    crate::structs::block::Block,
};

pub struct PoSConfig {
}

pub struct PoS {
}

impl ConsensusLayerTrait for PoS {
    fn initialize(&mut self) -> Result<(), Error> {
        unimplemented!("initialize not implemented for PoS");
    }

    fn create_block(&self) -> Result<Block, Error> {
        unimplemented!("create_block not implemented for PoS");
    }
}