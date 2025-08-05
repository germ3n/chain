use {
    crate::{
        db::{DatabaseTrait, rocksdb::TransactionRocksDB},
        crypto::keypair::Keypair,
        vm::{VMTrait, evm::EVM, lvm::LVM},
        crypto::hash::Hash,
        mempool::Mempool,
    },
    std::sync::{Arc, OnceLock, atomic::{AtomicU64}, RwLock},
};

pub struct Context<D: DatabaseTrait, VM: VMTrait + Send + Sync> {
    pub db: Arc<D>,
    pub miner_keypair: RwLock<Option<Keypair>>,
    pub block_height: AtomicU64,
    pub curr_top: RwLock<Hash>,
    pub curr_circulating_supply: RwLock<u128>,
    pub max_circulating_supply: RwLock<u128>,
    pub issued_supply: RwLock<u128>,
    pub total_burned_supply: RwLock<u128>,
    pub emission_rate: RwLock<u128>,
    pub vms: RwLock<Vec<Arc<VM>>>,
    pub mempool: Mempool,
}

impl<D: DatabaseTrait, VM: VMTrait + Send + Sync> Context<D, VM> {
    pub fn new(db: Arc<D>, vm: Option<Arc<VM>>) -> Self {
        Self { 
            db, 
            block_height: AtomicU64::new(0),
            miner_keypair: RwLock::new(None),
            curr_top: RwLock::new(Hash::null()),
            curr_circulating_supply: RwLock::new(0),
            max_circulating_supply: RwLock::new(0),
            issued_supply: RwLock::new(0),
            total_burned_supply: RwLock::new(0),
            emission_rate: RwLock::new(0),
            vms: RwLock::new(match vm {
                Some(vm) => vec![vm],
                None => vec![],
            }),
            mempool: Mempool::new(),
        }
    }

    pub fn calc_block_reward(&self) -> u128 {
        let mut block_reward = *self.emission_rate.read().unwrap();
        let mut half_supply = *self.max_circulating_supply.read().unwrap() / 2;
        let mut curr_issued_supply = *self.issued_supply.read().unwrap();

        while curr_issued_supply > half_supply {
            curr_issued_supply -= half_supply;
            block_reward /= 2;
            half_supply /= 2;
        }

        return block_reward;
    }
}

#[cfg(feature = "evm")]
impl<D: DatabaseTrait + Send + Sync + 'static> Context<D, EVM<D>> {
    pub fn new_with_evm(db: Arc<D>) -> Self {
        Self::new(db.clone(), Some(Arc::new(EVM::new(db.clone()))))
    }
}

#[cfg(feature = "lvm")]
impl<D: DatabaseTrait + Send + Sync> Context<D, LVM<D>> {
    pub fn new_with_lvm(db: Arc<D>) -> Self {
        Self::new(db.clone(), Some(Arc::new(LVM::new(db.clone()))))
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
    (vm, $index:expr) => {
        ctx!().vms.read().unwrap()[$index].as_ref().expect("VM not initialized")
    };
    ($key:ident) => {
        ctx!().$key
    };
}