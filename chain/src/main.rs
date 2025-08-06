#[macro_use] extern crate log;

pub mod config;
pub mod structs;
pub mod db;
pub mod context;
pub mod consensus;
pub mod crypto;
pub mod serialize;
pub mod vm;
pub mod mempool;

use {
    clap::Parser,
    crate::context::CTX,
    crate::db::{DatabaseTrait, rocksdb::TransactionRocksDB},
    crate::context::Context,
    crate::config::Config,
    crate::crypto::keypair::Keypair,
    std::sync::{Arc, atomic::Ordering},
    crate::serialize::json::{SerializableJSON}
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
        Context::new_with_evm(
            Arc::new(db), 
            Config::bitcoin()
        )
    });

    ctx!(miner_keypair).write().unwrap().replace(Keypair::new_ed25519().expect("Failed to generate keypair"));

    info!("Context initialized");
    info!("Block height: {}", ctx!(block_height).load(Ordering::Relaxed));

    info!("{}", ctx!(config).to_json().expect("Failed to serialize config"));
    std::process::exit(0);
    
    match consensus::main_loop() {
        Ok(_) => info!("Consensus loop finished"),
        Err(e) => error!("Consensus loop failed: {}", e),
    }
}