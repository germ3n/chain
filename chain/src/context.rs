use {
    crate::{
        config::Config, 
        crypto::{hash::{Hash}, keypair::Keypair}, 
        db::{DatabaseTrait, rocksdb::TransactionRocksDB}, 
        event::{EventManager, EventManagerTrait}, 
        mempool::Mempool, 
        vm::{evm::EVM, lvm::LVM, VMTrait},
        structs::block::Block,
    },
    cutedogs::document,
    std::sync::{atomic::{AtomicU32, AtomicU64, Ordering}, Arc, OnceLock, RwLock},
};

#[cfg(feature = "rpc-json")]
use crate::rpc::json::JsonRpc;

pub struct Context { // https://www.youtube.com/watch?v=x2QT3HciQN0
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
    pub event_manager: EventManager,
    pub db: RwLock<Arc<Box<TransactionRocksDB>>>,
    #[cfg(feature = "rpc-json")]
    pub rpc: JsonRpc,
}

impl Context {
    #[document(
        summary = "Creates a new blockchain context with specified virtual machine",
        params = {
            "vm": "Optional virtual machine instance for smart contract execution",
            "config": "Blockchain configuration including consensus parameters and network settings"
        },
        returns = "A fully initialized Context ready for blockchain operations"
    )]
    pub fn new(vm: Option<Arc<Box<dyn VMTrait + Send + Sync>>>, config: Config) -> Self {
        Self { 
            block_height: AtomicU64::new(0),
            miner_keypair: RwLock::new(None),
            curr_top: RwLock::new(Block::genesis().header.hash),
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
            event_manager: EventManager::new(),
            db: RwLock::new(Arc::new(Box::new(TransactionRocksDB::open(&[&"db".to_string()]).expect("Failed to open database")))),
            #[cfg(feature = "rpc-json")]
            rpc: JsonRpc::new(),
        }
    }

    #[document(
        summary = "Calculates the block reward based on the current block height",
        returns = "The calculated block reward"
    )]
    pub fn calc_block_reward(&self) -> u128 {
        self.config.calc_block_reward(self.block_height.load(Ordering::Relaxed))
    }
}

#[cfg(feature = "evm")]
impl Context {
    #[document(
        summary = "Creates a new context with an EVM virtual machine",
        params = {
            "config": "Blockchain configuration including consensus parameters and network settings"
        },
        returns = "A fully initialized Context with EVM support"
    )]
    pub fn new_with_evm(config: Config) -> Self {
        Self::new(Some(Arc::new(Box::new(EVM::new()))), config)
    }
}

#[cfg(feature = "lvm")]
impl Context {
    #[document(
        summary = "Creates a new context with an LVM virtual machine",
        params = {
            "config": "Blockchain configuration including consensus parameters and network settings"
        },
        returns = "A fully initialized Context with LVM support"
    )]
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
    (get) => {
        $crate::context::CTX.get()
    };
    ($key:ident) => {
        ctx!().$key
    };
}