#[macro_use] extern crate log;

pub mod r#const;
pub mod config;
pub mod structs;
pub mod db;
pub mod context;
pub mod consensus;
pub mod crypto;
pub mod serialize;
pub mod vm;
pub mod mempool;
pub mod util;
pub mod net;
pub mod rpc;
pub mod currency;
pub mod event;

use {
    clap::Parser,
    crate::context::CTX,
    crate::context::Context,
    crate::config::Config,
    crate::crypto::keypair::Keypair,
    std::sync::atomic::Ordering,
    crate::rpc::{RpcHost, RpcTrait},
    chain::document,
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    miner_pubkey: Option<String>,
}

#[document(summary = "Main entry point for the blockchain node", returns = "Does not return (runs until termination)")]
fn main() {
    pretty_env_logger::init();
    let _args = Args::parse();
    
    info!("Initializing context");
    CTX.get_or_init(|| Context::new_with_lvm(Config::bitcoin()));

    ctx!(miner_keypair).write().unwrap().replace(Keypair::new_ed25519().expect("Failed to generate keypair"));

    info!("Context initialized");
    info!("Block height: {}", ctx!(block_height).load(Ordering::Relaxed));

    #[cfg(feature = "rpc-json")]
    std::thread::spawn(|| {
        info!("Starting RPC server");
        ctx!(rpc).listen(RpcHost::Localhost, 8080).expect("Failed to listen");
    }); 
    
    match consensus::main_loop() {
        Ok(_) => info!("Consensus loop finished"),
        Err(e) => error!("Consensus loop failed: {}", e),
    }
}