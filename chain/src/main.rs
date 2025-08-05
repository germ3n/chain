#[macro_use] extern crate log;

pub mod structs;
pub mod db;
pub mod context;
pub mod consensus;
pub mod crypto;
pub mod serialize;

use {
    clap::Parser,
    crate::context::CTX,
    crate::db::{DatabaseTrait, rocksdb::TransactionRocksDB},
    crate::context::Context,
    std::sync::{Arc, atomic::Ordering},
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    miner_pubkey: Option<String>,
}

fn main() {
    pretty_env_logger::init();
    let _args = Args::parse();
    
    info!("Initializing context");
    CTX.get_or_init(|| {
        let db = TransactionRocksDB::open(&"db").expect("Failed to open database");
        Context::new(Arc::new(db))
    });

    info!("Context initialized");
    info!("Block height: {}", ctx!(block_height).load(Ordering::Relaxed));
    
    match consensus::main_loop() {
        Ok(_) => info!("Consensus loop finished"),
        Err(e) => error!("Consensus loop failed: {}", e),
    }
}