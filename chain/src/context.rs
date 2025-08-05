use {
    crate::{
        db::{DatabaseTrait, rocksdb::TransactionRocksDB},
        crypto::keypair::Keypair,
    },
    std::sync::{Arc, OnceLock, atomic::{AtomicU64}, RwLock},
};

pub struct Context<D: DatabaseTrait> {
    pub db: Arc<D>,
    pub miner_keypair: RwLock<Option<Keypair>>,
    pub block_height: AtomicU64,
    pub curr_circulating_supply: RwLock<u128>,
    pub max_circulating_supply: RwLock<u128>,
    pub issued_supply: RwLock<u128>,
    pub emission_rate: RwLock<u128>,
}

impl<D: DatabaseTrait> Context<D> {
    pub fn new(db: Arc<D>) -> Self {
        Self { 
            db, 
            block_height: AtomicU64::new(0),
            miner_keypair: RwLock::new(None),
            curr_circulating_supply: RwLock::new(0),
            max_circulating_supply: RwLock::new(0),
            issued_supply: RwLock::new(0),
            emission_rate: RwLock::new(0),
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

pub static CTX: OnceLock<Context<TransactionRocksDB>> = OnceLock::new();

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