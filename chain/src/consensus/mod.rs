use {
    crate::ctx,
    std::sync::atomic::Ordering,
    crate::structs::block::Block,
    crate::db::DatabaseTrait,
    anyhow::Error,
};

pub fn main_loop() -> Result<(), Error> {
    info!("Starting consensus loop");
    loop {
        let block_height = ctx!(block_height).load(Ordering::Relaxed);
        info!("Mining block height: {}", block_height);

        ctx!(db).with_transaction(|tx| {
            let _block = Block::genesis();
            let _ = tx.put_raw_bytes(b"block_height", &block_height.to_le_bytes());
            Ok(())
        })?;

        info!("Block mined, pubkey: {:?}", ctx!(miner_keypair).read().unwrap().as_ref().unwrap().get_pubkey().expect("Invalid keypair"));

        ctx!(block_height).store(block_height + 1, Ordering::Relaxed);
    }
}