//todo: implement the transaction stuff properly

use {
    super::{DatabaseTrait, TransactionTrait}, 
    crate::crypto::hash::{Hash, HashAlgorithm}, 
    anyhow::Error, 
    cutedogs::document, 
    rocksdb::{Options, Transaction as RocksDBTransaction, TransactionDB, TransactionDBOptions, DB}, 
    std::{any::Any, collections::HashMap, 
    sync::{Arc, RwLock}},
    crate::crypto::hash::{Hashable},
};

pub struct RocksDB {
    db: DB,
    path: String,
}

pub struct TransactionRocksDB {
    db: TransactionDB,
    path: String,
}

pub struct Transaction/*<'a>*/ {
    //tx: Option<RocksDBTransaction<'a, TransactionDB>>,
    changes: RwLock<HashMap<Hash, Vec<u8>>>,
}

impl DatabaseTrait for RocksDB {
    #[document(summary = "Opens a database", params = {
        "options": "The options to open the database with"
    }, returns = "The opened database")]
    fn open(options: &[&dyn Any]) -> Result<Self, Error> {
        let path = options[0].downcast_ref::<String>().ok_or(Error::msg("Invalid options"))?;
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(RocksDB { db, path: path.to_owned() })
    }

    #[document(summary = "Gets a raw bytes from the database", params = {
        "key": "The key to get the raw bytes from"
    }, returns = "The raw bytes")]
    fn get_raw_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let value = self.db.get(key)?;
        Ok(value)
    }

    #[document(summary = "Puts a raw bytes into the database", params = {
        "key": "The key to put the raw bytes into",
        "value": "The raw bytes to put into the database"
    }, returns = "The result of the operation")]
    fn put_raw_bytes(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        self.db.put(key, value)?;
        Ok(())
    }

    #[document(summary = "Executes a transaction", params = {
        "f": "The function to execute the transaction with"
    }, returns = "The result of the transaction")]
    fn with_transaction(&self, _f: impl FnOnce(&dyn TransactionTrait) -> Result<(), Error>) -> Result<(), Error> {
        unimplemented!("with_transaction not implemented")
    }
}

impl DatabaseTrait for TransactionRocksDB {
    #[document(summary = "Opens a transaction database", params = {
        "options": "The options to open the transaction database with"
    }, returns = "The opened transaction database")]
    fn open(options: &[&dyn Any]) -> Result<Self, Error> {
        let path = options[0].downcast_ref::<String>().ok_or(Error::msg("Invalid options"))?;
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let tx_opts = TransactionDBOptions::default();
        let db = TransactionDB::open(&opts, &tx_opts, path)?;
        Ok(TransactionRocksDB { db, path: path.to_owned() })
    }

    #[document(summary = "Gets a raw bytes from the transaction database", params = {
        "key": "The key to get the raw bytes from"
    }, returns = "The raw bytes")]
    fn get_raw_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let value = self.db.get(key)?;
        Ok(value)
    }

    #[document(summary = "Puts a raw bytes into the transaction database", params = {
        "key": "The key to put the raw bytes into",
        "value": "The raw bytes to put into the transaction database"
    }, returns = "The result of the operation")]
    fn put_raw_bytes(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        self.db.put(key, value)?;
        Ok(())
    }

    #[document(summary = "Executes a transaction", params = {
        "f": "The function to execute the transaction with"
    }, returns = "The result of the transaction")]
    fn with_transaction(&self, f: impl FnOnce(&dyn TransactionTrait) -> Result<(), Error>) -> Result<(), Error> {
        //let tx = self.db.transaction();
        let transaction = Transaction { changes: RwLock::new(HashMap::new()) };
        
        match f(&transaction) {
            Ok(()) => {
                //transaction.commit()?;
                Ok(())
            }
            Err(e) => {
                //let _ = transaction.rollback();
                Err(e)
            }
        }
    }
}

impl/*<'a>*/ TransactionTrait for Transaction/*<'a>*/ {
    #[document(summary = "Gets a raw bytes from the transaction", params = {
        "key": "The key to get the raw bytes from"
    }, returns = "The raw bytes")]
    fn get_raw_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        ///let tx = self.tx.as_ref().ok_or(Error::msg("Transaction already consumed"))?;
        //tx.get(key).map_err(Error::from)
        Ok(self.changes.read().unwrap().get(&key.hash(HashAlgorithm::Blake3_256)).cloned())
    }

    #[document(summary = "Puts a raw bytes into the transaction", params = {
        "key": "The key to put the raw bytes into",
        "value": "The raw bytes to put into the transaction"
    }, returns = "The result of the operation")]
    fn put_raw_bytes(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        //let tx = self.tx.as_ref().ok_or(Error::msg("Transaction already consumed"))?;
        //tx.put(key, value).map_err(Error::from)
        self.changes.write().unwrap().insert(key.hash(HashAlgorithm::Blake3_256), value.to_vec());
        Ok(())
    }

    #[document(summary = "Commits the transaction", returns = "The result of the commit")]
    fn commit(&mut self) -> Result<(), Error> {
        //let tx = self.tx.take().ok_or(Error::msg("Transaction already consumed"))?;
        //tx.commit().map_err(Error::from)
        Ok(())
    }   

    #[document(summary = "Rolls back the transaction", returns = "The result of the rollback")]
    fn rollback(&mut self) -> Result<(), Error> {
        //let tx = self.tx.take().ok_or(Error::msg("Transaction already consumed"))?;
        //tx.rollback().map_err(Error::from)
        Ok(())
    }
}