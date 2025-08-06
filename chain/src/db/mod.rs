#[cfg(feature = "rocksdb")]
pub mod rocksdb;

use {
    anyhow::Error,
    std::any::Any,
    crate::crypto::hash::{Hash, HashAlgorithm},
};

pub trait DatabaseTrait {
    fn open(options: &dyn Any) -> Result<Self, Error> where Self: Sized;
    fn get_raw_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error>;
    fn put_raw_bytes(&self, key: &[u8], value: &[u8]) -> Result<(), Error>;
    fn with_transaction<F>(&self, f: F) -> Result<(), Error>
    where F: FnOnce(&mut dyn TransactionTrait) -> Result<(), Error>;
}

pub trait TransactionTrait {
    fn get_raw_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error>;
    fn put_raw_bytes(&self, key: &[u8], value: &[u8]) -> Result<(), Error>;
    fn commit(&mut self) -> Result<(), Error>;
    fn rollback(&mut self) -> Result<(), Error>;
}

pub fn hash_key(salt: Option<&[u8]>, key: &[u8]) -> [u8; 32] {
    return match Hash::with_hasher(HashAlgorithm::Blake3_256, |hasher: &mut dyn std::io::Write| {
        if let Some(salt) = salt {
            hasher.write(salt).expect("Failed to write salt to hasher");
        }
        hasher.write(key).expect("Failed to write key to hasher");
    }) {
        Hash::Blake3_256(hash) => hash,
        _ => panic!("Failed to hash key"),
    };
}