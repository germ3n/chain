#[cfg(feature = "rocksdb")]
pub mod rocksdb;

use {
    anyhow::Error,
    std::any::Any,
    crate::crypto::hash::{Hash, HashAlgorithm},
    std::sync::Arc,
};

pub trait DatabaseTrait: Send + Sync {
    fn open(options: &[&dyn Any]) -> Result<Self, Error> where Self: Sized;
    fn get_raw_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error>;
    fn put_raw_bytes(&self, key: &[u8], value: &[u8]) -> Result<(), Error>;
    fn with_transaction(&self, f: impl FnOnce(&mut dyn TransactionTrait) -> Result<(), Error>) -> Result<(), Error>
    where Self: Sized;
}

pub trait TransactionTrait: Send {
    fn get_raw_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error>;
    fn put_raw_bytes(&self, key: &[u8], value: &[u8]) -> Result<(), Error>;
    fn commit(&mut self) -> Result<(), Error>;
    fn rollback(&mut self) -> Result<(), Error>;
}

pub fn hash_key(key: &[u8]) -> Hash {
    Hash::from_bytes(HashAlgorithm::Blake3_256, key)
}

pub fn hash_key_salted(salt: &[u8], key: &[u8]) -> Hash {
    Hash::with_hasher(HashAlgorithm::Blake3_256, |hasher: &mut dyn std::io::Write| {
        hasher.write(salt).expect("Failed to write salt to hasher");
        hasher.write(key).expect("Failed to write key to hasher");
    })
}

pub fn open_db(path: String) -> Result<Arc<Box<dyn DatabaseTrait>>, Error> {
    let path: &'static str = Box::leak(path.into_boxed_str());
    Ok(Arc::new(Box::new(rocksdb::TransactionRocksDB::open(&[&path])?)))
}