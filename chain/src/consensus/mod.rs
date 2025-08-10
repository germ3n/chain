pub mod pow;
pub mod pos;

use {
    crate::ctx,
    std::sync::atomic::Ordering,
    crate::structs::block::Block,
    anyhow::Error,
    crate::crypto::hash::{Hashable, HashAlgorithm},
    crate::event::{Event, EventManagerTrait},
    crate::serialize::binary::SerializableBytes,
    crate::db::DatabaseTrait,
    std::sync::Arc,
};

pub trait ConsensusLayerTrait {
    fn initialize(&mut self) -> Result<(), Error>;
    fn start_consensus(&self) -> Result<(), Error>;
}

pub fn main_loop() -> Result<(), Error> {
    info!("Initializing VM");
    ctx!(vms).write().unwrap().iter_mut().for_each(|vm| {
        Arc::get_mut(vm).expect("Failed to get VM").initialize(&[
            &"_accounts".to_string(), 
            &"_contracts".to_string(), 
            &"_transactions".to_string()
        ]).expect("Failed to initialize VM")
    });
    info!("VM initialized");

    info!("Starting consensus loop");
    loop {
        let block_height = ctx!(block_height).load(Ordering::Relaxed);
        info!("Mining block height: {}", block_height);

        ctx!(mempool).for_each_tx(|hash, tx| {
            info!("Mempool tx: 0x{}: {}", hash, tx);
        });

        let mut block = Block::coinbase(ctx!(curr_top).read().unwrap().clone());
        block.validate()?;

        ctx!(db).write().unwrap().with_transaction(|tx| {
            let _difficulty = ctx!(difficulty).load(Ordering::Relaxed);

            tx.put_raw_bytes(b"block_height", block_height.to_bytes()?.as_slice())?;
            block.txs[0].sign(ctx!(miner_keypair).read().unwrap().as_ref().expect("Invalid keypair"))?;

            for vm in ctx!(vms).read().unwrap().iter() {
                vm.prepare_execution()?;
                for tx in block.txs.iter().skip(1) { // skip coinbase tx
                    ctx!(event_manager).add_event(Event::NewTx(tx.hash(HashAlgorithm::Blake3_256)));

                    vm.start_tx()?;
                    match vm.execute_tx(tx) {
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
                vm.finalize_execution()?;
            }
            
            Ok(())
        })?;

        ctx!(event_manager).add_event(Event::NewBlock(block.hash(HashAlgorithm::Blake3_256)));

        info!("Block mined, pubkey: {}: {}", ctx!(miner_keypair).read().unwrap().as_ref().expect("Invalid keypair").pubkey().expect("Invalid pubkey"), block);
        ctx!(block_height).store(block_height + 1, Ordering::Relaxed);

        ctx!(mempool).remove_txs(block.txs.iter().skip(1).map(|tx| tx.hash(HashAlgorithm::Blake3_256)).collect());
    }
}