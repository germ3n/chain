use {
    crate::{
        crypto::keypair::Keypair,
        vm::{VMTrait, evm::EVM, lvm::LVM},
        crypto::hash::Hash,
        mempool::Mempool,
        config::Config,
    },
    std::sync::{Arc, OnceLock, atomic::{AtomicU64, AtomicU32, Ordering}, RwLock},
};

#[cfg(feature = "rpc-json")]
use crate::rpc::json::JsonRpc;

pub struct Context {
    pub miner_keypair: RwLock<Option<Keypair>>,
    pub block_height: AtomicU64,
    pub curr_top: RwLock<Hash>,
    pub curr_circulating_supply: RwLock<u128>,
    pub issued_supply: RwLock<u128>,
    pub total_burned_supply: RwLock<u128>,
    pub difficulty: AtomicU32,
    pub vms: RwLock<Vec<Arc<Box<dyn VMTrait + Send + Sync>>>>,
    pub mempool: Mempool,
    pub config: Config,
    #[cfg(feature = "rpc-json")]
    pub rpc: JsonRpc,
}

impl Context {
    pub fn new(vm: Option<Arc<Box<dyn VMTrait + Send + Sync>>>, config: Config) -> Self {
        Self { 
            block_height: AtomicU64::new(0),
            miner_keypair: RwLock::new(None),
            curr_top: RwLock::new(Hash::null()),
            curr_circulating_supply: RwLock::new(0),
            issued_supply: RwLock::new(0),
            total_burned_supply: RwLock::new(0),
            difficulty: AtomicU32::new(0),
            vms: RwLock::new(match vm {
                Some(vm) => vec![vm],
                None => vec![],
            }),
            mempool: Mempool::new(),
            config,
            #[cfg(feature = "rpc-json")]
            rpc: JsonRpc::new(),
        }
    }

    pub fn calc_block_reward(&self) -> u128 {
        self.config.calc_block_reward(self.block_height.load(Ordering::Relaxed))
    }
}

#[cfg(feature = "evm")]
impl Context {
    pub fn new_with_evm(config: Config) -> Self {
        Self::new(Some(Arc::new(Box::new(EVM::new()))), config)
    }
}

#[cfg(feature = "lvm")]
impl Context {
    pub fn new_with_lvm(config: Config) -> Self {
        Self::new(Some(Arc::new(Box::new(LVM::new()))), config)
    }
}

pub static CTX: OnceLock<Context> = OnceLock::new();

#[macro_export]
macro_rules! ctx {
    () => {
        $crate::context::CTX.get().expect("Context not initialized")
    };
    ($key:ident) => {
        ctx!().$key
    };
}