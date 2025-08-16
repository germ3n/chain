use {
    crate::consensus::ConsensusLayerTrait,
    anyhow::Error,
    crate::structs::block::Block,
    cutedogs::document,
};

pub struct PoSConfig {
}

pub struct PoS {
}

impl ConsensusLayerTrait for PoS {
    #[document(summary = "Initializes the PoS consensus layer", returns = "The result of the initialization")]
    fn initialize(&mut self) -> Result<(), Error> {
        unimplemented!("initialize not implemented for PoS");
    }

    #[document(summary = "Creates a block for the PoS consensus layer", returns = "The created block")]
    fn create_block(&self) -> Result<Block, Error> {
        unimplemented!("create_block not implemented for PoS");
    }
}