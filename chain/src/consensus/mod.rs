use {
    crate::ctx,
    std::sync::atomic::Ordering,
    crate::structs::block::Block,
    crate::db::DatabaseTrait,
    anyhow::Error,
    crate::vm::VMTrait,
    crate::crypto::hash::{Hashable, HashAlgorithm},
    crate::serialize::to_bytes,
};

pub fn main_loop() -> Result<(), Error> {
    info!("Initializing VM");
    ctx!(vms).read().unwrap().iter().for_each(|vm| vm.init().expect("Failed to initialize VM"));
    info!("VM initialized");

    info!("Starting consensus loop");
    loop {
        let block_height = ctx!(block_height).load(Ordering::Relaxed);
        info!("Mining block height: {}", block_height);

        ctx!(mempool).for_each_tx(|hash, tx| {
            info!("Mempool tx: 0x{}: {}", hash, tx);
        });

        let block = Block::coinbase(
            ctx!(curr_top).read().unwrap().clone(), 
            crate::crypto::keypair::PublicKey::Ed25519([0; 32]) //ctx!(miner_keypair).read().unwrap().as_ref().expect("Invalid keypair").get_pubkey().expect("Invalid pubkey")
        );

        ctx!(db).with_transaction(|tx| {
            tx.put_raw_bytes(b"block_height", to_bytes(&block_height)?.as_slice())?;

            for vm in ctx!(vms).read().unwrap().iter() {
                vm.prepare_execution()?;
                for tx in block.txs.iter().skip(1) { // skip coinbase tx
                    vm.start_tx()?;
                    match vm.execute_tx(tx) {
                        Ok(_) => {
                            info!("Tx 0x{} executed successfully", tx.hash(HashAlgorithm::Blake3_256));
                            vm.commit_tx()?;
                        },
                        Err(_) => {
                            error!("Tx 0x{} execution failed", tx.hash(HashAlgorithm::Blake3_256));
                            vm.rollback_tx()?;
                        },
                    }
                }
                vm.finalize_execution()?;
            }
            
            Ok(())
        })?;

        info!("Block mined, pubkey: {}", crate::crypto::keypair::PublicKey::Ed25519([0; 32]));//ctx!(miner_keypair).read().unwrap().as_ref().expect("Invalid keypair").get_pubkey().expect("Invalid pubkey"));
        ctx!(block_height).store(block_height + 1, Ordering::Relaxed);

        ctx!(mempool).remove_txs(block.txs.iter().map(|tx| tx.hash(HashAlgorithm::Blake3_256)).collect());
    }
}