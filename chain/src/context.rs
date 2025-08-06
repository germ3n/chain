use {
    crate::{
        db::{DatabaseTrait, rocksdb::TransactionRocksDB},
        crypto::keypair::Keypair,
        vm::{VMTrait, evm::EVM, lvm::LVM},
        crypto::hash::Hash,
        mempool::Mempool,
        config::Config,
    },
    std::sync::{Arc, OnceLock, atomic::{AtomicU64, Ordering}, RwLock},
};

pub struct Context<D: DatabaseTrait, VM: VMTrait<D> + Send + Sync> {
    pub db: Arc<D>,
    pub miner_keypair: RwLock<Option<Keypair>>,
    pub block_height: AtomicU64,
    pub curr_top: RwLock<Hash>,
    pub curr_circulating_supply: RwLock<u128>,
    pub issued_supply: RwLock<u128>,
    pub total_burned_supply: RwLock<u128>,
    pub vms: RwLock<Vec<Arc<VM>>>,
    pub mempool: Mempool,
    pub config: Config,
}

impl<D: DatabaseTrait, VM: VMTrait<D> + Send + Sync> Context<D, VM> {
    pub fn new(db: Arc<D>, vm: Option<Arc<VM>>, config: Config) -> Self {
        Self { 
            db, 
            block_height: AtomicU64::new(0),
            miner_keypair: RwLock::new(None),
            curr_top: RwLock::new(Hash::null()),
            curr_circulating_supply: RwLock::new(0),
            issued_supply: RwLock::new(0),
            total_burned_supply: RwLock::new(0),
            vms: RwLock::new(match vm {
                Some(vm) => vec![vm],
                None => vec![],
            }),
            mempool: Mempool::new(),
            config,
        }
    }

    pub fn calc_block_reward(&self) -> u128 {
        self.config.calc_block_reward(self.block_height.load(Ordering::Relaxed))
    }
}

#[cfg(feature = "evm")]
impl<D: DatabaseTrait + Send + Sync + 'static> Context<D, EVM<D>> {
    pub fn new_with_evm(db: Arc<D>, config: Config) -> Self {
        Self::new(db.clone(), Some(Arc::new(EVM::new(db.clone()))), config)
    }
}

#[cfg(feature = "lvm")]
impl<D: DatabaseTrait + Send + Sync> Context<D, LVM<D>> {
    pub fn new_with_lvm(db: Arc<D>, config: Config) -> Self {
        Self::new(db.clone(), Some(Arc::new(LVM::new(db.clone()))), config)
    }
}

pub static CTX: OnceLock<Context<TransactionRocksDB, EVM<TransactionRocksDB>>> = OnceLock::new();

#[macro_export]
macro_rules! ctx {
    () => {
        $crate::context::CTX.get().expect("Context not initialized")
    };
    (db) => {
        ctx!().db.as_ref()
    };
    ($key:ident) => {
        ctx!().$key
    };
}