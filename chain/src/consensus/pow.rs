use {
    crate::consensus::ConsensusLayerTrait,
    anyhow::Error,
    crate::crypto::hash::HashAlgorithm,
    crate::structs::block::Block,
    std::sync::atomic::Ordering,
    log::info,
    crate::ctx,
    cutedogs::document,
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

        let block = Block::coinbase(ctx!(curr_top).read().unwrap().clone());
        block.validate()?;

        /*ctx!(db).write().unwrap().with_transaction(|tx| {
            let _difficulty = ctx!(difficulty).load(Ordering::Relaxed);

            tx.put_raw_bytes(b"block_height", block_height.to_bytes()?.as_slice())?;
            block.txs[0].sign(ctx!(miner_keypair).read().unwrap().as_ref().expect("Invalid keypair"))?;

            for vm in ctx!(vms).read().unwrap().iter() {
                vm.prepare_execution()?;
                for batch in ctx!(mempool).get_tx_batches() {
                    for tx in batch {
                        ctx!(event_manager).add_event(Event::NewTx(tx.hash(HashAlgorithm::Blake3_256)));

                        vm.start_tx()?;
                        match vm.execute_tx(&tx) {
                            Ok(_) => {
                                info!("Tx 0x{} executed successfully", tx.hash(HashAlgorithm::Blake3_256));
                                vm.commit_tx()?;
                            },
                            Err(e) => {
                                error!("Tx 0x{} execution failed: {}", tx.hash(HashAlgorithm::Blake3_256), e);
                                vm.rollback_tx()?;
                            },
                        }
                    }
                }
                vm.finalize_execution()?;
            }

            Ok(())
        })?;*/

        Ok(block)
    }
}