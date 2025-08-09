use {
    super::{DatabaseTrait, TransactionTrait},
    anyhow::Error,
    rocksdb::{DB, TransactionDB, Options, TransactionDBOptions, Transaction as RocksDBTransaction},
    std::any::Any,
};

pub struct RocksDB {
    db: DB,
}

pub struct TransactionRocksDB {
    db: TransactionDB,
}

pub struct Transaction<'a> {
    tx: Option<RocksDBTransaction<'a, TransactionDB>>,
}

impl DatabaseTrait for RocksDB {
    fn open(options: &[&dyn Any]) -> Result<Self, Error> {
        let path = options[0].downcast_ref::<&str>().ok_or(Error::msg("Invalid options"))?;
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(RocksDB { db })
    }

    fn get_raw_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let value = self.db.get(key)?;
        Ok(value)
    }

    fn put_raw_bytes(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        self.db.put(key, value)?;
        Ok(())
    }

    fn with_transaction(&self, _f: impl FnOnce(&mut dyn TransactionTrait) -> Result<(), Error>) -> Result<(), Error> {
        unimplemented!("with_transaction not implemented")
    }
}

impl DatabaseTrait for TransactionRocksDB {
    fn open(options: &[&dyn Any]) -> Result<Self, Error> {
        let path = options[0].downcast_ref::<&str>().ok_or(Error::msg("Invalid options"))?;
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let tx_opts = TransactionDBOptions::default();
        let db = TransactionDB::open(&opts, &tx_opts, path)?;
        Ok(TransactionRocksDB { db })
    }

    fn get_raw_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let value = self.db.get(key)?;
        Ok(value)
    }

    fn put_raw_bytes(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        self.db.put(key, value)?;
        Ok(())
    }

    fn with_transaction(&self, f: impl FnOnce(&mut dyn TransactionTrait) -> Result<(), Error>) -> Result<(), Error> {
        let tx = self.db.transaction();
        let mut transaction = Transaction { tx: Some(tx) };
        
        match f(&mut transaction) {
            Ok(()) => {
                transaction.commit()?;
                Ok(())
            }
            Err(e) => {
                let _ = transaction.rollback();
                Err(e)
            }
        }
    }
}

impl<'a> TransactionTrait for Transaction<'a> {
    fn get_raw_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let tx = self.tx.as_ref().ok_or(Error::msg("Transaction already consumed"))?;
        let value = tx.get(key)?;
        Ok(value)
    }

    fn put_raw_bytes(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let tx = self.tx.as_ref().ok_or(Error::msg("Transaction already consumed"))?;
        tx.put(key, value).map_err(Error::from)?;
        Ok(())
    }

    fn commit(&mut self) -> Result<(), Error> {
        let tx = self.tx.take().ok_or(Error::msg("Transaction already consumed"))?;
        tx.commit().map_err(Error::from)?;
        Ok(())
    }   

    fn rollback(&mut self) -> Result<(), Error> {
        let tx = self.tx.take().ok_or(Error::msg("Transaction already consumed"))?;
        tx.rollback().map_err(Error::from)?;
        Ok(())
    }
}